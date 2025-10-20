use leptos::*;
use leptos::html::{Div, Textarea};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use uuid::Uuid;
use std::sync::Arc;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;
use pulldown_cmark::{Parser, Options, Event, Tag, CodeBlockKind, TagEnd};
use web_sys;

use crate::{
    api::ApiClient,
    auth::use_auth,
    components::DocumentSidebar,
    models::{Document, DocumentSummary},
    app::{THEME_LIGHT, THEME_DARK, KROKI_URL, use_chat_sidebar, EditorContext, use_editor, use_sidebar},
};

#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    // Redirect if not logged in
    Effect::new(move |_| {
        if !auth.state.get().loading && auth.state.get().user.is_none() {
            navigate("/login", Default::default());
        }
    });

    // Signal for mobile sidebar
    let sidebar_open = RwSignal::new(false);

    view! {
        <Show
            when=move || !auth.state.get().loading && auth.state.get().user.is_some()
            fallback=move || {
                if auth.state.get().loading {
                    view! { <div class="min-h-screen flex items-center justify-center"><div class="text-lg">"Loading..."</div></div> }
                } else {
                    view! { <div class="min-h-screen flex items-center justify-center"><div class="text-lg">"Redirecting to login..."</div></div> }
                }
            }
        >
            {move || {
                let auth_state = auth.state.get();
                let user = auth_state.user.unwrap();
                let token = auth_state.token.unwrap();

                let documents = RwSignal::new(Vec::<DocumentSummary>::new());
                let selected_document = RwSignal::new(Option::<Document>::None);
                let loading_documents = RwSignal::new(true);
                let error_message = RwSignal::new(Option::<String>::None);

                let client = Arc::new(ApiClient::with_token(token.clone()));
                
                let client_effect = client.clone();
                Effect::new(move |_| {
                    let client_effect_clone = client_effect.clone();
                    spawn_local(async move {
                        match client_effect_clone.get_documents().await {
                            Ok(docs) => {
                                documents.set(docs);
                                loading_documents.set(false);
                            },
                            Err(err) => {
                                error_message.set(Some(err.error));
                                loading_documents.set(false);
                            }
                        }
                    });
                });

                let client_action = client.clone();
                let create_document = Action::new_local(move |title: &String| {
                    let title = title.clone();
                    let client_action = client_action.clone();
                    async move {
                        match client_action.create_document(&title, Some("# New Document\n\nStart writing...")).await {
                            Ok(doc) => {
                                documents.update(|docs| {
                                    docs.insert(0, DocumentSummary {
                                        id: doc.id,
                                        title: doc.title.clone(),
                                        created_at: doc.created_at,
                                        updated_at: doc.updated_at,
                                    });
                                });
                                selected_document.set(Some(doc));
                                Ok(())
                            }
                            Err(err) => Err(err),
                        }
                    }
                });

                let client_for_editor = client.clone();
                let client_for_on_select = client.clone();

                view! {
                    <div class="relative flex h-screen bg-gray-100 dark:bg-gray-800">
                        <DocumentSidebar
                            documents=documents.read_only().into()
                            selected_document=selected_document.read_only().into()
                            loading=loading_documents.read_only().into()
                            on_select=move |doc_id| {
                                let client = client_for_on_select.clone();
                                spawn_local(async move {
                                    match client.get_document(doc_id).await {
                                        Ok(doc) => selected_document.set(Some(doc)),
                                        Err(err) => error_message.set(Some(err.error)),
                                    }
                                });
                            }
                            on_create=move |title| { create_document.dispatch(title); }
                            on_logout=move || { auth.logout.dispatch(()); }
                            on_theme=move || {
                                // Make sure we have a user to update
                                if let Some(mut current_user) = auth.state.get().user {
                                    // Toggle the theme on the user object
                                    let current_theme = current_user.theme.clone();
                                    current_user.theme = if current_theme == THEME_LIGHT {
                                        THEME_DARK.to_string()
                                    } else {
                                        THEME_LIGHT.to_string()
                                    };

                                    // Dispatch the action with the updated user data
                                    auth.update_settings.dispatch(current_user);
                                }
                            }
                            user_name=user.username.clone()
                        />

                        <Show when=move || sidebar_open.get()>
                            <div
                                class="fixed inset-0 bg-gray-900 bg-opacity-50 z-20 md:hidden"
                                on:click=move |_| sidebar_open.set(false)
                            ></div>
                        </Show>

                        <main class="flex-1 flex flex-col overflow-hidden">
                            <Show
                                when=move || selected_document.get().is_some()
                                fallback=move || view! {
                                    <div class="flex-1 flex items-center justify-center text-gray-500">
                                        <div class="text-center">
                                            <h2 class="text-2xl font-semibold mb-2">"Welcome to Dr. Markdown"</h2>
                                            <p>"Select a document from the sidebar or create a new one to get started."</p>
                                        </div>
                                    </div>
                                }
                            >
                                <DocumentEditor
                                    document=selected_document.get().unwrap()
                                    on_save=move |updated_doc| {
                                        selected_document.set(Some(updated_doc.clone()));
                                        documents.update(|docs| {
                                            if let Some(doc_summary) = docs.iter_mut().find(|d| d.id == updated_doc.id) {
                                                doc_summary.title = updated_doc.title.clone();
                                                doc_summary.updated_at = updated_doc.updated_at;
                                            }
                                        });
                                    }
                                    on_delete=move |doc_id| {
                                        documents.update(|docs| {
                                            docs.retain(|d| d.id != doc_id);
                                        });
                                        selected_document.set(None);
                                    }
                                    client=client_for_editor.clone()
                                />
                            </Show>
                             {move || error_message.get().map(|msg| view! {
                                <div class="bg-red-50 border-l-4 border-red-400 p-4 m-4">
                                    <div class="flex">
                                        <div class="ml-3">
                                            <p class="text-sm text-red-700">{msg}</p>
                                        </div>
                                    </div>
                                </div>
                            })}
                        </main>
                    </div>
                }
            }}
        </Show>
    }
}

// Helper function to create a Kroki URL
fn generate_kroki_url(diagram_type: &str, code: &str) -> String {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(code.as_bytes()).unwrap();
    let compressed_bytes = encoder.finish().unwrap();
    let encoded_data = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, compressed_bytes);
    format!("{}/{}/svg/{}", KROKI_URL, diagram_type, encoded_data)
}

pub fn render_markdown(markdown_content: &str) -> String {

    let parser = Parser::new_ext(markdown_content, Options::all());

    let supported_diagrams = ["mermaid", "plantuml", "graphviz", "ditaa", "blockdiag", "structurizr", "seqdiag"];
    let mut in_diagram_block = false;
    let mut diagram_lang = String::new();

    // The event transformation logic remains the same.
    let transformed_events: Vec<Event> = parser.filter_map(|event| {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                let lang_str = lang.into_string();
                if supported_diagrams.contains(&lang_str.as_str()) {
                    in_diagram_block = true;
                    diagram_lang = lang_str;
                    None 
                } else {
                    Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang_str.into()))))
                }
            }
            Event::Text(text) => {
                if in_diagram_block {
                    let kroki_url = generate_kroki_url(&diagram_lang, &text);
                    let html = format!("<img class=\"kroki-diagram\" src=\"{}\" alt=\"Diagram: {}\">", kroki_url, diagram_lang);
                    
                    Some(Event::Html(html.into()))
                } else {
                    Some(Event::Text(text))
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_diagram_block {
                    in_diagram_block = false;
                    diagram_lang.clear();
                    None
                } else {
                    Some(Event::End(TagEnd::CodeBlock))
                }
            }
            _ => Some(event),
        }
    }).collect();

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, transformed_events.into_iter());

    html_output
}

#[component]
pub fn DocumentEditor(
    document: Document,
    on_save: impl Fn(Document) + 'static + Clone,
    on_delete: impl Fn(Uuid) + 'static + Clone,
    client: Arc<ApiClient>,
) -> impl IntoView {

    let (content, set_content) = signal(document.content.clone());

    let (title, set_title) = signal(document.title.clone());
    let (is_editing, set_is_editing) = signal(false);
    let (saving, set_saving) = signal(false);
    let (show_confirm_dialog, set_show_confirm_dialog) = signal(false);
    let show_preview = RwSignal::new(false);

    let chat_sidebar = use_chat_sidebar();
    let mobile_sidebar = use_sidebar();

    // For scrolling the preview pane
    let editor_ref = create_node_ref::<Textarea>();
    let preview_ref = create_node_ref::<Div>();

    let client_save = client.clone();
    let doc_id = document.id;
    let save_document = Action::new_local(move |_: &()| {
        let new_title = title.get_untracked();
        let new_content = content.get_untracked();
        let client_save_clone = client_save.clone();
        
        async move {
            set_saving.set(true);
            let result = client_save_clone.update_document(doc_id, Some(&new_title), Some(&new_content)).await;
            set_saving.set(false);
            result
        }
    });

    let on_save_clone = on_save.clone();
    Effect::new(move |_| {
        save_document.value().with(|opt_result| {
            if let Some(Ok(updated_doc)) = opt_result {
                on_save_clone(updated_doc.clone());
            }
        });
    });

    let client_delete = client.clone();
    let delete_document = Action::new_local(move |_: &()| {
        let client_delete_clone = client_delete.clone();
        async move {
            client_delete_clone.delete_document(doc_id).await
        }
    });

    let on_delete_clone = on_delete.clone();
    Effect::new(move |_| {
        delete_document.value().with(|opt_result| {
            if let Some(Ok(_)) = opt_result {
                on_delete_clone(doc_id);
            }
        });
    });

    let rendered_html = move || {
        render_markdown(&content.get())
    };

    view! {
<div class="flex-1 flex flex-col overflow-hidden">
            <header class="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 px-4 sm:px-6 py-4 flex items-center justify-between">
                <div class="flex items-center flex-1 min-w-0">
                    <button
                        class="text-gray-500 dark:text-gray-400 focus:outline-none md:hidden mr-4"
                        on:click=move |_| mobile_sidebar.0.update(|open| *open = !*open)
                    >
                        // Hamburger icon SVG
                    </button>
                    <input
                        class="text-xl font-semibold text-gray-900 dark:text-gray-50 border-none outline-none bg-transparent w-full"
                        prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev))
                        on:blur=move |_| { save_document.dispatch(()); }
                    />
                </div>
                <div class="flex items-center space-x-2">
                    // This button enters edit mode
                    <Show when=move || !is_editing.get()>
                        <button
                            class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 border border-gray-300 rounded-md hover:bg-gray-200"
                            on:click=move |_| set_is_editing.update(|editing| *editing = !*editing)
                        >
                            Edit
                        </button>
                    </Show>

                    // The "Live Preview" button is now only shown when in Edit Mode
                    <Show when=move || is_editing.get()>
                        <button
                            class="px-4 py-2 text-sm font-medium rounded-md border"
                            class:bg-blue-50=move || show_preview.get()
                            on:click=move |_| show_preview.update(|show| *show = !*show)
                        >
                            "Live Preview"
                        </button>
                    </Show>

                    <button
                        class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 border border-gray-300 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        on:click=move |_| chat_sidebar.0.update(|open| *open = !*open)>
                        {move || if chat_sidebar.0.get() { "Close AI" } else { "AI Chat" }}
                    </button>
                    <button 
                        class="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                        on:click=move |_| { save_document.dispatch(()); }
                        disabled=move || { saving.get() || !is_editing.get() }

                        >
                        {move || if saving.get() { "Saving..." } else { "Save" }}
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-medium text-white bg-red-600 border border-transparent rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500" 
                        on:click=move |_| set_show_confirm_dialog.set(true)>
                        "Delete"
                    </button>
                </div>
            </header>

            <Show when=move || show_confirm_dialog.get()>
                <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50 flex items-center justify-center">
                    <div class="relative p-5 border w-96 shadow-lg rounded-md bg-white">
                        <div class="mt-3 text-center">
                            <h3 class="text-lg leading-6 font-medium text-gray-900">"Confirm Deletion"</h3>
                            <div class="mt-2 px-7 py-3">
                                <p class="text-sm text-gray-500">"Are you sure you want to delete this document? This action cannot be undone."</p>
                            </div>
                            <div class="items-center px-4 py-3">
                                <button
                                    on:click=move |_| {
                                        delete_document.dispatch(());
                                        set_show_confirm_dialog.set(false);
                                    }
                                    class="px-4 py-2 bg-red-600 text-white text-base font-medium rounded-md w-full shadow-sm hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500"
                                >
                                    "Delete"
                                </button>
                                <button
                                    on:click=move |_| { set_show_confirm_dialog.set(false); }
                                    class="mt-2 px-4 py-2 bg-gray-200 text-gray-700 text-base font-medium rounded-md w-full shadow-sm hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-500"
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            <div class="flex-1 overflow-hidden">
                <Show
                    when=move || is_editing.get()
                    fallback=move || view! {
                        // VIEW MODE: a single, full-pane preview.
                        <div class="h-full overflow-auto p-6 prose prose-lg max-w-none" inner_html=rendered_html()></div>
                    }
                >
                    // EDIT MODE: split-pane editor with preview.
                    <div class="flex flex-col h-full">
                        <Show when=move || show_preview.get()>
                            <div
                                class="resizable-pane border-b border-gray-200 dark:border-gray-700"
                                style="height: 50%;" // An initial height
                            >
                                <div 
                                    node_ref=preview_ref
                                    class="h-full overflow-auto p-6 prose prose-lg max-w-none dark:prose-invert" inner_html=rendered_html()>
                                </div>
                            </div>
                        </Show>
                        <div class="flex-1">
                            <textarea
                                node_ref=editor_ref
                                class="w-full h-full p-6 border-none outline-none resize-none font-mono text-sm dark:bg-gray-800 dark:text-gray-50"
                                prop:value=content
                                on:input=move |ev| {
                                    let new_value = event_target_value(&ev);
                                    set_content.set(new_value);

                                    // Defer the scroll calculation to the next animation frame.
                                    // This ensures the preview pane has been updated with the new HTML
                                    // before we try to measure its height for scrolling.
                                    request_animation_frame(move || {
                                        if let (Some(editor), Some(preview)) = (editor_ref.get_untracked(), preview_ref.get_untracked()) {
                                            // Cast the generic elements to their specific HTML types
                                            let editor_el: &web_sys::HtmlTextAreaElement = &editor;
                                            let preview_el: &web_sys::HtmlElement = &preview;

                                            // Get the cursor's position in the textarea
                                            let cursor_pos = editor_el.selection_start().unwrap_or(Some(0)).unwrap_or(0) as f64;
                                            let content_len = editor_el.value().len() as f64;

                                            if content_len > 0.0 {
                                                // Calculate how far down the document the cursor is (a ratio from 0.0 to 1.0)
                                                let scroll_ratio = cursor_pos / content_len;
                                                
                                                // Calculate the total scrollable height of the preview pane
                                                let max_preview_scroll = (preview_el.scroll_height() - preview_el.client_height()) as f64;
                                                
                                                // Determine the target scroll position based on the cursor's ratio
                                                let target_scroll_top = scroll_ratio * max_preview_scroll;
                                                
                                                // Set the preview pane's scroll position
                                                preview_el.set_scroll_top(target_scroll_top as i32);
                                            }
                                        }
                                    });
                                }
                                placeholder="Start writing your markdown..."
                            ></textarea>
                        </div>
                    </div>
                </Show>
            </div>
        </div>



    }
}