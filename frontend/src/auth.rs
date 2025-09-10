use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use leptos::logging::log;

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
    pub update_settings: Action<User, Result<User, String>>,
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

    let update_settings = Action::new_local(move |updated_user: &User| {
        let user_to_update = updated_user.clone();
        let set_state = set_state; // Capture the WriteSignal for state updates

        async move {
            let token = state.get_untracked().token;
            if let Some(token) = token {
                let client = ApiClient::with_token(token);

                match client.update_user_settings(&user_to_update.theme).await {
                    Ok(saved_user) => {
                        // Update localStorage
                        let _ = LocalStorage::set(USER_KEY, &saved_user);

                        // Update the global state
                        set_state.update(|s| {
                            log!("State updated! New theme is: {}", saved_user.theme);
                            s.user = Some(saved_user.clone());
                        });

                        Ok(saved_user)
                    }
                    Err(e) => Err(e.error),
                }
            } else {
                // If for some reason there's no token, return an error immediately.
                Err("Authentication token not found.".to_string())
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
        update_settings,
    };

    provide_context(auth_context);

    children()
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext not found")
}