use leptos::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use leptos_router::components::A;

use crate::auth::use_auth;
use crate::app::APP_BASE;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);

    // Redirect to home when user is logged in (covers both initial load and post-login)
    {
        let navigate = navigate.clone();
        Effect::new(move |_| {
            let state = auth.state.get();
            if !state.loading && state.user.is_some() {
                navigate("/", NavigateOptions { replace: true, ..Default::default() });
            }
        });
    }

    // Only use login action result for error display, not navigation
    Effect::new(move |_| {
        if let Some(Err(err)) = auth.login.value().get() {
            set_error_message.set(Some(err));
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_message.set(None);
        auth.login.dispatch((username.get(), password.get()));
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-sm w-full">
                <div class="text-center mb-8">
                    <div class="inline-flex items-center justify-center w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-xl mb-4">
                        <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                        </svg>
                    </div>
                    <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-50">"Sign in"</h2>
                    <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                        "Or "
                        <A href=format!("{}/register", APP_BASE) attr:class="font-medium text-blue-600 dark:text-blue-400 hover:text-blue-500">
                            "create a new account"
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
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded-lg text-sm">
                            {msg}
                        </div>
                    })}

                    <button
                        type="submit"
                        class="w-full py-2.5 px-4 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:focus:ring-offset-gray-900 disabled:opacity-50 transition-colors"
                        disabled=move || auth.login.pending().get()
                    >
                        {move || if auth.login.pending().get() { "Signing in..." } else { "Sign in" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
