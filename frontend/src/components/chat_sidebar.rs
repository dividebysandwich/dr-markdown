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

    let on_submit = move |_| {
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
                                                last_msg.text.push_str(&parsed.response);
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
        <aside class=move || format!(
            "w-80 bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-700 flex flex-col \
            fixed inset-y-0 right-0 z-50 transform {} transition-transform duration-300 ease-in-out",
            if chat_sidebar.0.get() { "right-0" } else { "-right-80" }
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