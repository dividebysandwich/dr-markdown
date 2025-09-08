use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

use crate::{api::ApiClient, models::User};

const TOKEN_KEY: &str = "auth_token";
const USER_KEY: &str = "auth_user";

#[derive(Debug, Clone)]
pub struct AuthState {
    pub user: Option<User>,
    pub token: Option<String>,
    pub loading: bool,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            user: None,
            token: None,
            loading: true,
        }
    }
}

#[derive(Clone)]
pub struct AuthContext {
    pub state: ReadSignal<AuthState>,
    pub login: Action<(String, String), Result<(), String>>,
    pub register: Action<(String, String), Result<(), String>>,
    pub logout: Action<(), ()>,
}

#[component]
pub fn AuthProvider(children: ChildrenFn) -> impl IntoView {
    let (state, set_state) = signal(AuthState::default());

    // Load token and user from localStorage on startup
    Effect::new(move |_| {
        let token: Option<String> = LocalStorage::get(TOKEN_KEY).ok();
        let user: Option<User> = LocalStorage::get(USER_KEY).ok();

        if token.is_some() && user.is_some() {
            set_state.update(|s| {
                s.token = token;
                s.user = user;
                s.loading = false;
            });
        } else {
            set_state.update(|s| {
                s.loading = false;
            });
        }
    });

    let login = Action::new_local(move |(username, password): &(String, String)| {
        let username = username.clone();
        let password = password.clone();
        
        async move {
            let client = ApiClient::new();
            match client.login(&username, &password).await {
                Ok(auth_response) => {
                    // Store in localStorage
                    let _ = LocalStorage::set(TOKEN_KEY, &auth_response.token);
                    let _ = LocalStorage::set(USER_KEY, &auth_response.user);

                    set_state.update(|s| {
                        s.token = Some(auth_response.token);
                        s.user = Some(auth_response.user);
                    });

                    Ok(())
                }
                Err(e) => Err(e.error),
            }
        }
    });

    let register = Action::new_local(move |(username, password): &(String, String)| {
        let username = username.clone();
        let password = password.clone();
        
        async move {
            let client = ApiClient::new();
            match client.register(&username, &password).await {
                Ok(auth_response) => {
                    // Store in localStorage
                    let _ = LocalStorage::set(TOKEN_KEY, &auth_response.token);
                    let _ = LocalStorage::set(USER_KEY, &auth_response.user);

                    set_state.update(|s| {
                        s.token = Some(auth_response.token);
                        s.user = Some(auth_response.user);
                    });

                    Ok(())
                }
                Err(e) => Err(e.error),
            }
        }
    });

    let logout = Action::new_local(move |_: &()| {
        async move {
            let _ = LocalStorage::delete(TOKEN_KEY);
            let _ = LocalStorage::delete(USER_KEY);

            set_state.update(|s| {
                s.token = None;
                s.user = None;
            });
        }
    });

    let auth_context = AuthContext {
        state: state,
        login,
        register,
        logout,
    };

    provide_context(auth_context);

    children()
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext not found")
}