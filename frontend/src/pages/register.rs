use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::components::A;

use crate::auth::use_auth;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);

    // Redirect if already logged in
    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if auth.state.get().user.is_some() {
                navigate("/", Default::default());
            }
        }
    });

    // Handle register result
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

        // Validation
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
        <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                <div>
                    <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
                        "Create your account"
                    </h2>
                    <p class="mt-2 text-center text-sm text-gray-600">
                        "Or "
                        <A href="/login" attr:class="font-medium text-blue-600 hover:text-blue-500">
                            "sign in to existing account"
                        </A>
                    </p>
                </div>
                
                <form class="mt-8 space-y-6" on:submit=on_submit>
                    <div class="space-y-4">
                        <div>
                            <label for="username" class="block text-sm font-medium text-gray-700">"Username"</label>
                            <input
                                id="username"
                                name="username"
                                type="text"
                                required
                                class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                placeholder="Enter username"
                                prop:value=username
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label for="password" class="block text-sm font-medium text-gray-700">"Password"</label>
                            <input
                                id="password"
                                name="password"
                                type="password"
                                required
                                class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                placeholder="Enter password"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label for="confirm_password" class="block text-sm font-medium text-gray-700">"Confirm Password"</label>
                            <input
                                id="confirm_password"
                                name="confirm_password"
                                type="password"
                                required
                                class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                placeholder="Confirm password"
                                prop:value=confirm_password
                                on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
                            {msg}
                        </div>
                    })}

                    <div>
                        <button
                            type="submit"
                            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
                            disabled=move || auth.register.pending().get()
                        >
                            {move || if auth.register.pending().get() { "Creating account..." } else { "Create account" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}