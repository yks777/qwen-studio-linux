use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};
use webkit2gtk::{
    CookieManagerExt, JavascriptResult, WebViewExt, WebsiteDataManagerExt,
};
use gio::Cancellable;

use crate::profile::manager::{self, CookieData, Session};

fn policy_to_i32(policy: soup::SameSitePolicy) -> i32 {
    use glib::translate::IntoGlib;
    policy.into_glib()
}

fn policy_from_i32(value: i32) -> soup::SameSitePolicy {
    match value {
        1 => soup::SameSitePolicy::Lax,
        2 => soup::SameSitePolicy::Strict,
        _ => soup::SameSitePolicy::None,
    }
}

fn cookie_to_data(cookie: &mut soup::Cookie) -> CookieData {
    CookieData {
        name: cookie.name().map(|s| s.to_string()).unwrap_or_default(),
        value: cookie.value().map(|s| s.to_string()).unwrap_or_default(),
        domain: cookie.domain().map(|s| s.to_string()),
        path: cookie.path().map(|s| s.to_string()),
        expires: cookie.expires().map(|dt| dt.to_unix()),
        secure: cookie.is_secure(),
        http_only: cookie.is_http_only(),
        same_site: policy_to_i32(cookie.same_site_policy()),
        max_age: -1,
    }
}

fn build_cookie(data: &CookieData) -> soup::Cookie {
    let domain = data.domain.clone().unwrap_or_default();
    let path = data.path.clone().unwrap_or_default();
    let mut cookie = soup::Cookie::new(&data.name, &data.value, &domain, &path, -1);
    if let Some(domain) = &data.domain {
        cookie.set_domain(domain);
    }
    if let Some(path) = &data.path {
        cookie.set_path(path);
    }
    cookie.set_secure(data.secure);
    cookie.set_http_only(data.http_only);
    if let Some(ts) = data.expires {
        if let Ok(dt) = glib::DateTime::from_unix_utc(ts) {
            cookie.set_expires(&dt);
        }
    }
    cookie.set_same_site_policy(policy_from_i32(data.same_site));
    cookie
}

pub async fn capture_session(app: &AppHandle, window_label: &str, profile_id: &str) {
    let window = match app.get_webview_window(window_label) {
        Some(w) => w,
        None => return,
    };

    let (tx_cookies, rx_cookies) = tokio::sync::oneshot::channel::<Vec<CookieData>>();
    let (tx_ls, rx_ls) = tokio::sync::oneshot::channel::<Option<String>>();

    let result = window.with_webview(move |any| {
        let web_view = any.inner();

        let uri = web_view
            .uri()
            .map(|u| u.to_string())
            .unwrap_or_else(|| manager::PROFILE_MAIN_URL.to_string());

        if let Some(manager) = web_view
            .website_data_manager()
            .and_then(|m| m.cookie_manager())
        {
            let tx_cookies = tx_cookies;
            manager.cookies(
                uri.as_str(),
                None::<&Cancellable>,
                move |res: Result<Vec<soup::Cookie>, glib::Error>| {
                    let cookies = res.unwrap_or_default();
                    let data: Vec<CookieData> = cookies
                        .into_iter()
                        .map(|mut c| cookie_to_data(&mut c))
                        .collect();
                    let _ = tx_cookies.send(data);
                },
            );
        } else {
            let _ = tx_cookies.send(Vec::new());
        }

        let tx_ls = tx_ls;
        #[allow(deprecated)]
        web_view.run_javascript(
            "JSON.stringify(Object.entries(localStorage))",
            None::<&Cancellable>,
            move |res: Result<JavascriptResult, glib::Error>| {
                let ls = res
                    .ok()
                    .and_then(|r| r.js_value())
                    .map(|v| {
                        use javascriptcore::ValueExt;
                        v.to_str().to_string()
                    });
                let _ = tx_ls.send(ls);
            },
        );
    });

    if result.is_err() {
        return;
    }

    let cookies = rx_cookies.await.unwrap_or_default();
    let local_storage = rx_ls
        .await
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Vec<[String; 2]>>(&raw).ok())
        .map(|entries| {
            entries
                .into_iter()
                .map(|p| (p[0].clone(), p[1].clone()))
                .collect::<std::collections::HashMap<String, String>>()
        })
        .unwrap_or_default();

    let session = Session {
        cookies,
        local_storage,
    };
    let _ = manager::save_session(profile_id, &session);
}

pub async fn restore_session(app: &AppHandle, window_label: &str, profile_id: &str) {
    let session = match manager::load_session(profile_id) {
        Some(s) if !s.cookies.is_empty() => s,
        _ => return,
    };
    let window = match app.get_webview_window(window_label) {
        Some(w) => w,
        None => return,
    };

    let cookies = session.cookies;
    let total = cookies.len();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let state = Arc::new(Mutex::new((0usize, Some(tx))));

    let result = window.with_webview(move |any| {
        let web_view = any.inner();

        let manager = match web_view
            .website_data_manager()
            .and_then(|m| m.cookie_manager())
        {
            Some(m) => m,
            None => {
                let mut g = state.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(tx) = g.1.take() {
                    let _ = tx.send(());
                }
                return;
            }
        };

        for cookie_data in &cookies {
            let mut cookie = build_cookie(cookie_data);
            let state = state.clone();
            manager.add_cookie(&mut cookie, None::<&Cancellable>, move |_res: Result<(), glib::Error>| {
                let mut g = state.lock().unwrap_or_else(|e| e.into_inner());
                g.0 += 1;
                if g.0 >= total {
                    if let Some(tx) = g.1.take() {
                        let _ = tx.send(());
                    }
                }
            });
        }

        if total == 0 {
            let mut g = state.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(tx) = g.1.take() {
                let _ = tx.send(());
            }
        }
    });

    if result.is_ok() {
        let _ = rx.await;
    }
}

pub fn restore_local_storage_js(session: &Session) -> String {
    let pairs: Vec<[String; 2]> = session
        .local_storage
        .iter()
        .map(|(k, v)| [k.clone(), v.clone()])
        .collect();
    let json = serde_json::to_string(&pairs).unwrap_or_else(|_| "[]".to_string());
    format!(
        "(function(){{var __items={};for(var i=0;i<__items.length;i++){{try{{localStorage.setItem(__items[i][0],__items[i][1]);}}catch(e){{}}}}}})();",
        json
    )
}
