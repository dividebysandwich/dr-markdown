use leptos::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::components::A;

use crate::auth::use_auth;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);

    // Redirect if already logged in
    {
        let navigate = navigate.clone();
        Effect::new(move |_| {
            if auth.state.get().user.is_some() {
                navigate("/", Default::default());
            }
        });
    }

    // Handle login result
    {
        let navigate = navigate.clone();
        Effect::new(move |_| {
            if let Some(result) = auth.login.value().get() {
                match result {
                    Ok(_) => {
                        navigate("/", Default::default());
                    }
                    Err(err) => {
                        set_error_message.set(Some(err));
                    }
                }
            }
        });
    }

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_message.set(None);
        auth.login.dispatch((username.get(), password.get()));
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8">
                <div>
                    <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
                        "Sign in to your account"
                    </h2>
                    <p class="mt-2 text-center text-sm text-gray-600">
                        "Or "
                        <A href="/register" attr:class="font-medium text-blue-600 hover:text-blue-500">
                            "create a new account"
                        </A>
                    </p>
                </div>
                
                <form class="mt-8 space-y-6" on:submit=on_submit>
                    <div class="rounded-md shadow-sm -space-y-px">
                        <div>
                            <label for="username" class="sr-only">"Username"</label>
                            <input
                                id="username"
                                name="username"
                                type="text"
                                required
                                class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                                placeholder="Username"
                                prop:value=username
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label for="password" class="sr-only">"Password"</label>
                            <input
                                id="password"
                                name="password"
                                type="password"
                                required
                                class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                                placeholder="Password"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
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
                            disabled=move || auth.login.pending().get()
                        >
                            {move || if auth.login.pending().get() { "Signing in..." } else { "Sign in" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}