use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::components::A;

use crate::auth::use_auth;
use crate::app::APP_BASE;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);

    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if auth.state.get().user.is_some() {
                navigate("/", Default::default());
            }
        }
    });

    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if let Some(result) = auth.register.value().get() {
                match result {
                    Ok(_) => {
                        navigate("/", Default::default());
                    }
                    Err(err) => {
                        set_error_message.set(Some(err));
                    }
                }
            }
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_error_message.set(None);

        let username_val = username.get();
        let password_val = password.get();
        let confirm_password_val = confirm_password.get();

        if username_val.len() < 3 {
            set_error_message.set(Some("Username must be at least 3 characters long".to_string()));
            return;
        }

        if password_val.len() < 6 {
            set_error_message.set(Some("Password must be at least 6 characters long".to_string()));
            return;
        }

        if password_val != confirm_password_val {
            set_error_message.set(Some("Passwords do not match".to_string()));
            return;
        }

        auth.register.dispatch((username_val, password_val));
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-sm w-full">
                <div class="text-center mb-8">
                    <div class="inline-flex items-center justify-center w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-xl mb-4">
                        <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z"></path>
                        </svg>
                    </div>
                    <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-50">"Create account"</h2>
                    <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                        "Or "
                        <A href=format!("{}/login", APP_BASE) attr:class="font-medium text-blue-600 dark:text-blue-400 hover:text-blue-500">
                            "sign in to existing account"
                        </A>
                    </p>
                </div>

                <form class="space-y-4" on:submit=on_submit>
                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 space-y-4">
                        <div>
                            <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Username"</label>
                            <input
                                id="username"
                                name="username"
                                type="text"
                                required
                                class="w-full px-3 py-2.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Enter username"
                                prop:value=username
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Password"</label>
                            <input
                                id="password"
                                name="password"
                                type="password"
                                required
                                class="w-full px-3 py-2.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Enter password"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label for="confirm_password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Confirm Password"</label>
                            <input
                                id="confirm_password"
                                name="confirm_password"
                                type="password"
                                required
                                class="w-full px-3 py-2.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Confirm password"
                                prop:value=confirm_password
                                on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded-lg text-sm">
                            {msg}
                        </div>
                    })}

                    <button
                        type="submit"
                        class="w-full py-2.5 px-4 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:focus:ring-offset-gray-900 disabled:opacity-50 transition-colors"
                        disabled=move || auth.register.pending().get()
                    >
                        {move || if auth.register.pending().get() { "Creating account..." } else { "Create account" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
