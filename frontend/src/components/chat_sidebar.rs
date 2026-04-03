use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::app::{use_chat_sidebar, use_editor};
use crate::api::{ApiClient, ChatApiRequest};
use crate::auth::use_auth;
use futures::stream::TryStreamExt;
use js_sys;
use wasm_streams::ReadableStream;


#[derive(serde::Deserialize, Clone)]
pub struct OllamaStreamResponse {
    pub response: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ChatMessage {
    sender: String,
    text: String,
}

#[component]
pub fn ChatSidebar() -> impl IntoView {
    let chat_sidebar = use_chat_sidebar();
    let editor = use_editor();
    let auth = use_auth();

    let messages = RwSignal::new(Vec::<ChatMessage>::new());
    let user_input = RwSignal::new(String::new());
    let is_thinking = RwSignal::new(false);

    let do_submit = move || {
        let message = user_input.get_untracked();
        if message.is_empty() { return; }

        let context = editor.0.get_untracked();
        messages.update(|msgs| msgs.push(ChatMessage { sender: "User".to_string(), text: message.clone() }));
        user_input.set("".to_string());

        messages.update(|msgs| msgs.push(ChatMessage { sender: "AI".to_string(), text: "".to_string() }));
        is_thinking.set(true);

        spawn_local(async move {
            let body = ChatApiRequest { context, message };
            let token = auth.state.get_untracked().token;

            if let Some(token) = token {
                let client = ApiClient::with_token(token);
                let res = client.ollama_chat_streaming(&body).await;

                if let Ok(res) = res {
                    let raw_stream = res.body().unwrap();
                    let mut stream = ReadableStream::from_raw(raw_stream).into_stream();

                    loop {
                        match stream.try_next().await {
                            Ok(Some(chunk)) => {
                                let decoder = web_sys::TextDecoder::new().unwrap();
                                let uint8_array = js_sys::Uint8Array::from(chunk);
                                let mut vec = vec![0u8; uint8_array.length() as usize];
                                uint8_array.copy_to(&mut vec[..]);
                                let chunk_str = decoder.decode_with_u8_array(&vec[..]).unwrap();

                                for line in chunk_str.split('\n').filter(|s| !s.is_empty()) {
                                    if let Ok(parsed) = serde_json::from_str::<OllamaStreamResponse>(line) {
                                        messages.update(|msgs| {
                                            if let Some(last_msg) = msgs.last_mut() {
                                                last_msg.text.push_str(&parsed.response.replace("\n", "<br/>"));
                                            }
                                        });
                                    }
                                }
                            }
                            Ok(None) => {
                                break;
                            }
                            Err(e) => {
                                messages.update(|msgs| {
                                    if let Some(last_msg) = msgs.last_mut() {
                                        last_msg.text = format!("Error: {:?}", e);
                                    }
                                });
                                break;
                            }
                        }
                    }
                } else {
                     messages.update(|msgs| {
                        if let Some(last_msg) = msgs.last_mut() {
                            last_msg.text = "Error: Could not connect to the server.".to_string();
                        }
                    });
                }
            }
            is_thinking.set(false);
        });
    };

    view! {
        // Backdrop on mobile
        <Show when=move || chat_sidebar.0.get()>
            <div
                class="fixed inset-0 bg-black/40 backdrop-blur-sm z-40 sm:hidden"
                on:click=move |_| chat_sidebar.0.set(false)
            ></div>
        </Show>

        <aside class=move || format!(
            "w-full sm:w-80 bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700 flex flex-col \
            fixed inset-y-0 right-0 z-50 transform {} transition-transform duration-300 ease-in-out \
            sm:max-w-80",
            if chat_sidebar.0.get() { "translate-x-0" } else { "translate-x-full" }
        )>
            <header class="px-4 py-3 flex items-center justify-between border-b border-gray-200 dark:border-gray-700 shrink-0">
                <button
                    class="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                    on:click=move |_| chat_sidebar.0.set(false)
                    title="Close Assistant"
                >
                    <svg class="w-5 h-5 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                    </svg>
                </button>
                <h2 class="text-sm font-semibold text-gray-800 dark:text-gray-100">"AI Assistant"</h2>
                <div class="w-8"></div>
            </header>

            <div class="flex-1 overflow-y-auto p-4 space-y-3">
                <Show when=move || messages.get().is_empty()>
                    <div class="text-center py-8">
                        <svg class="w-10 h-10 mx-auto text-gray-300 dark:text-gray-600 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z"></path>
                        </svg>
                        <p class="text-xs text-gray-400 dark:text-gray-500">"Ask the AI about your document"</p>
                    </div>
                </Show>
                <For
                    each=move || messages.get()
                    key=|msg| format!("{}-{}", msg.sender, msg.text)
                    children=move |msg| {
                        let is_user = Memo::new(move |_| msg.sender == "User");
                        view! {
                            <div class=move || if is_user.get() { "text-right" } else { "text-left" }>
                                <div class="inline-block p-3 rounded-xl max-w-[85%] text-sm"
                                     class=("bg-blue-600", move || is_user.get())
                                     class=("text-white", move || is_user.get())
                                     class=("bg-gray-100", move || !is_user.get())
                                     class=("dark:bg-gray-700", move || !is_user.get())
                                     class=("text-gray-800", move || !is_user.get())
                                     class=("dark:text-gray-200", move || !is_user.get())
                                >
                                    <div inner_html={msg.text}></div>
                                </div>
                            </div>
                        }
                    }
                />
                <Show when=move || is_thinking.get()>
                    <div class="text-left">
                        <div class="inline-block p-3 rounded-xl bg-gray-100 dark:bg-gray-700">
                            <div class="flex items-center space-x-1.5">
                                <div class="w-2 h-2 bg-gray-400 dark:bg-gray-500 rounded-full animate-bounce" style="animation-delay: 0ms"></div>
                                <div class="w-2 h-2 bg-gray-400 dark:bg-gray-500 rounded-full animate-bounce" style="animation-delay: 150ms"></div>
                                <div class="w-2 h-2 bg-gray-400 dark:bg-gray-500 rounded-full animate-bounce" style="animation-delay: 300ms"></div>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>

            <div class="p-3 border-t border-gray-200 dark:border-gray-700 shrink-0">
                <div class="flex gap-2">
                    <textarea
                        class="flex-1 p-2.5 text-sm border border-gray-200 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                        rows="2"
                        placeholder="Ask about your document..."
                        prop:value=user_input
                        on:input=move |ev| user_input.set(event_target_value(&ev))
                        on:keypress=move |ev| {
                            if ev.key() == "Enter" && !ev.shift_key() {
                                ev.prevent_default();
                                do_submit();
                            }
                        }
                    ></textarea>
                    <button
                        class="self-end p-2.5 text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors shrink-0"
                        on:click=move |_| { do_submit(); }
                        disabled=move || is_thinking.get()
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
                        </svg>
                    </button>
                </div>
            </div>
        </aside>
    }
}
