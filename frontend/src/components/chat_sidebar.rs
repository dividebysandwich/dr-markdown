// In frontend/src/components/chat_sidebar.rs

use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::app::{use_chat_sidebar, use_editor};
use crate::api::ApiClient;
use crate::auth::use_auth;

#[derive(Clone, Debug, PartialEq)]
struct ChatMessage {
    sender: String,
    text: String,
}

#[component]
pub fn ChatSidebar() -> impl IntoView {
    let sidebar = use_chat_sidebar();
    let editor = use_editor();
    let auth = use_auth();

    let messages = RwSignal::new(Vec::<ChatMessage>::new());
    let user_input = RwSignal::new(String::new());
    let is_thinking = RwSignal::new(false);

    // --- FIX #1: Replace `Action` with a manual "trigger" signal ---
    // This signal will hold the data needed for the async task.
    let send_trigger = RwSignal::new(None::<(String, String)>);

    // This Effect will run our async code whenever the trigger is set.
    Effect::new(move |_| {
        // Track the trigger signal
        if let Some((context, message)) = send_trigger.get() {
            // Use spawn_local for non-`Send` futures
            spawn_local(async move {
                is_thinking.set(true);
                let token = auth.state.get_untracked().token;
                
                if let Some(token) = token {
                    let client = ApiClient::with_token(token.clone());
                    match client.ollama_chat(&context, &message).await {
                        Ok(reply) => {
                            messages.update(|msgs| msgs.push(ChatMessage { sender: "AI".to_string(), text: reply }));
                        },
                        Err(e) => {
                            messages.update(|msgs| msgs.push(ChatMessage { sender: "AI".to_string(), text: format!("Error: {}", e.error) }));
                        }
                    }
                } else {
                    messages.update(|msgs| msgs.push(ChatMessage { sender: "AI".to_string(), text: "Error: Not authenticated.".to_string() }));
                }
                is_thinking.set(false);
            });
            // Reset the trigger so it can be fired again
            send_trigger.set(None);
        }
    });

    let on_submit = move |_| {
        let message = user_input.get_untracked();
        if message.is_empty() { return; }

        messages.update(|msgs| msgs.push(ChatMessage { sender: "User".to_string(), text: message.clone() }));
        
        let context = editor.0.get_untracked();

        send_trigger.set(Some((context, message)));
        
        user_input.set("".to_string());
    };

    view! {
        <aside class=move || format!(
            "w-96 bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-700 flex flex-col \
            fixed inset-y-0 right-0 z-30 transform {} transition-transform duration-300 ease-in-out",
            if sidebar.0.get() { "translate-x-0" } else { "translate-x-full" }
        )>
            <header class="p-4 border-b border-gray-200 dark:border-gray-700">
                <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-100">AI Assistant</h2>
            </header>
            
            <div class="flex-1 overflow-y-auto p-4 space-y-4">
                <For
                    each=move || messages.get()
                    key=|msg| format!("{}-{}", msg.sender, msg.text)
                    children=move |msg| {
                        let is_user = Memo::new(move |_| msg.sender == "User");
                        view! {
                            <div class=move || if is_user.get() { "text-right" } else { "text-left" }>
                                <div class="inline-block p-3 rounded-lg"
                                     class:bg-blue-500=move || is_user.get()
                                     class:text-white=move || is_user.get()
                                     class:bg-gray-200=move || !is_user.get()
                                     class:dark:bg-gray-700=move || !is_user.get()
                                >
                                    {msg.text}
                                </div>
                            </div>
                        }
                    }
                />
                <Show when=move || is_thinking.get()>
                    <p class="text-sm text-gray-500 italic">"AI is thinking..."</p>
                </Show>
            </div>

            <div class="p-4 border-t border-gray-200 dark:border-gray-700">
                <textarea
                    class="w-full p-2 border border-gray-300 rounded-md dark:bg-gray-800 dark:border-gray-600"
                    rows="3"
                    prop:value=user_input
                    on:input=move |ev| user_input.set(event_target_value(&ev))
                ></textarea>
                <button
                    class="w-full mt-2 px-4 py-2 text-white bg-blue-600 rounded-md hover:bg-blue-700"
                    on:click=on_submit
                    disabled=move || is_thinking.get()
                >
                    "Send"
                </button>
            </div>
        </aside>
    }
}