use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use uuid::Uuid;
use std::sync::Arc;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;
use pulldown_cmark::{Parser, Options, Event, Tag, CodeBlockKind, TagEnd};
use web_sys;
use js_sys;

use crate::{
    api::ApiClient,
    auth::use_auth,
    components::DocumentSidebar,
    models::{Document, DocumentSummary},
    app::{THEME_LIGHT, THEME_DARK, KROKI_URL, APP_BASE, use_chat_sidebar, use_sidebar, use_editor, use_dirty},
};

#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if !auth.state.get().loading && auth.state.get().user.is_none() {
            navigate("/login", NavigateOptions { replace: true, ..Default::default() });
        }
    });

    let sidebar_open = RwSignal::new(false);

    view! {
        <Show
            when=move || !auth.state.get().loading && auth.state.get().user.is_some()
            fallback=move || {
                if auth.state.get().loading {
                    view! { <div class="min-h-screen flex items-center justify-center"><div class="animate-spin rounded-full h-10 w-10 border-b-2 border-blue-600"></div></div> }
                } else {
                    view! { <div class="min-h-screen flex items-center justify-center"><div class="text-gray-500 dark:text-gray-400">"Redirecting to login..."</div></div> }
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
                let editor_context = use_editor();
                let dirty = use_dirty();

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
                                dirty.0.set(false);
                                Ok(())
                            }
                            Err(err) => Err(err),
                        }
                    }
                });

                let client_for_editor = client.clone();
                let client_for_on_select = client.clone();

                view! {
                    <div class="relative flex h-full bg-gray-50 dark:bg-gray-900">
                        <DocumentSidebar
                            documents=documents.read_only().into()
                            selected_document=selected_document.read_only().into()
                            loading=loading_documents.read_only().into()
                            on_select=move |doc_id| {
                                let client = client_for_on_select.clone();
                                spawn_local(async move {
                                    match client.get_document(doc_id).await {
                                        Ok(doc) => {
                                            selected_document.set(Some(doc));
                                            dirty.0.set(false);
                                        },
                                        Err(err) => error_message.set(Some(err.error)),
                                    }
                                });
                            }
                            on_create=move |title| { create_document.dispatch(title); }
                            on_logout=move || { auth.logout.dispatch(()); }
                            on_theme=move || {
                                if let Some(mut current_user) = auth.state.get().user {
                                    let current_theme = current_user.theme.clone();
                                    current_user.theme = if current_theme == THEME_LIGHT {
                                        THEME_DARK.to_string()
                                    } else {
                                        THEME_LIGHT.to_string()
                                    };
                                    auth.update_settings.dispatch(current_user);
                                }
                            }
                            user_name=user.username.clone()
                        />

                        <Show when=move || sidebar_open.get()>
                            <div
                                class="fixed inset-0 bg-black/40 z-20 md:hidden"
                                on:click=move |_| sidebar_open.set(false)
                            ></div>
                        </Show>

                        <main class="flex-1 flex flex-col overflow-hidden">
                            <Show
                                when=move || selected_document.get().is_some()
                                fallback=move || view! {
                                    <div class="flex-1 flex items-center justify-center p-8">
                                        <div class="text-center max-w-md">
                                            <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                                            </svg>
                                            <h2 class="text-xl font-semibold text-gray-700 dark:text-gray-300 mb-2">"Welcome to Dr. Markdown"</h2>
                                            <p class="text-gray-500 dark:text-gray-400 text-sm">"Select a document from the sidebar or create a new one to get started."</p>
                                        </div>
                                    </div>
                                }
                            >
                                <DocumentEditor
                                    document=selected_document.get().unwrap()
                                    on_save=move |updated_doc| {
                                        selected_document.set(Some(updated_doc.clone()));
                                        dirty.0.set(false);
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
                                        editor_context.0.set(String::new());
                                        dirty.0.set(false);
                                    }
                                    client=client_for_editor.clone()
                                />
                            </Show>
                             {move || error_message.get().map(|msg| view! {
                                <div class="bg-red-50 dark:bg-red-900/20 border-l-4 border-red-400 p-4 m-4 rounded-r-lg">
                                    <p class="text-sm text-red-700 dark:text-red-400">{msg}</p>
                                </div>
                            })}
                        </main>
                    </div>
                }
            }}
        </Show>
    }
}

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

    // Share state
    let (share_token, set_share_token) = signal(document.share_token.clone());
    let (show_share_dialog, set_show_share_dialog) = signal(false);
    let (share_copied, set_share_copied) = signal(false);

    let chat_sidebar = use_chat_sidebar();
    let mobile_sidebar = use_sidebar();
    let dirty = use_dirty();

    let editor_context = use_editor();
    editor_context.0.set(document.content.clone());
    dirty.0.set(false);

    let editor_ref = NodeRef::new();
    let preview_ref = NodeRef::new();

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

    // Share actions
    let client_share = client.clone();
    let create_share = Action::new_local(move |_: &()| {
        let client_clone = client_share.clone();
        async move {
            client_clone.create_share_link(doc_id).await
        }
    });

    Effect::new(move |_| {
        create_share.value().with(|opt_result| {
            if let Some(Ok(doc)) = opt_result {
                set_share_token.set(doc.share_token.clone());
            }
        });
    });

    let client_unshare = client.clone();
    let remove_share = Action::new_local(move |_: &()| {
        let client_clone = client_unshare.clone();
        async move {
            client_clone.remove_share_link(doc_id).await
        }
    });

    Effect::new(move |_| {
        remove_share.value().with(|opt_result| {
            if let Some(Ok(_doc)) = opt_result {
                set_share_token.set(None);
            }
        });
    });

    let rendered_html = move || {
        render_markdown(&content.get())
    };

    let share_url = move || {
        share_token.get().map(|token| {
            let window = web_sys::window().expect("no window");
            let location = window.location();
            let origin = location.origin().unwrap_or_default();
            format!("{}{}/shared/{}", origin, APP_BASE, token)
        })
    };

    view! {
        <div class="flex-1 flex flex-col overflow-hidden">
            // Toolbar
            <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-3 sm:px-4 py-2.5 shrink-0">
                // Title row
                <div class="flex items-center gap-2 mb-2">
                    <button
                        class="p-1.5 rounded-lg text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 md:hidden shrink-0 transition-colors"
                        on:click=move |_| mobile_sidebar.0.update(|open| *open = !*open)
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                        </svg>
                    </button>
                    <input
                        class="flex-1 text-lg font-semibold text-gray-900 dark:text-gray-50 border-none outline-none bg-transparent min-w-0"
                        prop:value=title
                        on:input=move |ev| {
                            set_title.set(event_target_value(&ev));
                            dirty.0.set(true);
                        }
                        on:blur=move |_| { save_document.dispatch(()); }
                    />
                    // Dirty indicator
                    <Show when=move || dirty.0.get()>
                        <span class="text-xs text-amber-500 dark:text-amber-400 shrink-0 font-medium">"Unsaved"</span>
                    </Show>
                </div>
                // Action buttons row — wraps on mobile
                <div class="flex items-center gap-1.5 flex-wrap">
                    <Show when=move || !is_editing.get()>
                        <button
                            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            on:click=move |_| set_is_editing.set(true)
                        >
                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                            </svg>
                            "Edit"
                        </button>
                    </Show>

                    <Show when=move || is_editing.get()>
                        <button
                            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            on:click=move |_| set_is_editing.set(false)
                        >
                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"></path>
                            </svg>
                            "View"
                        </button>
                    </Show>

                    <Show when=move || is_editing.get()>
                        <button
                            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-colors"
                            class=("bg-blue-50", move || show_preview.get())
                            class=("dark:bg-blue-900/30", move || show_preview.get())
                            class=("text-blue-700", move || show_preview.get())
                            class=("dark:text-blue-300", move || show_preview.get())
                            class=("text-gray-700", move || !show_preview.get())
                            class=("dark:text-gray-300", move || !show_preview.get())
                            class=("bg-gray-100", move || !show_preview.get())
                            class=("dark:bg-gray-700", move || !show_preview.get())
                            class=("hover:bg-gray-200", move || !show_preview.get())
                            class=("dark:hover:bg-gray-600", move || !show_preview.get())
                            on:click=move |_| show_preview.update(|show| *show = !*show)
                        >
                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z"></path>
                            </svg>
                            "Preview"
                        </button>
                    </Show>

                    <button
                        class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                        on:click=move |_| chat_sidebar.0.update(|open| *open = !*open)
                    >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z"></path>
                        </svg>
                        <span class="hidden sm:inline">{move || if chat_sidebar.0.get() { "Close AI" } else { "AI Chat" }}</span>
                    </button>

                    // Share button
                    <button
                        class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                        on:click=move |_| set_show_share_dialog.set(true)
                    >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"></path>
                        </svg>
                        <span class="hidden sm:inline">"Share"</span>
                    </button>

                    <div class="flex-1"></div>

                    <Show when=move || is_editing.get()>
                        <button
                            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors"
                            on:click=move |_| { save_document.dispatch(()); }
                            disabled=move || saving.get()
                        >
                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"></path>
                            </svg>
                            {move || if saving.get() { "Saving..." } else { "Save" }}
                        </button>
                    </Show>

                    <button
                        class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/40 transition-colors"
                        on:click=move |_| set_show_confirm_dialog.set(true)
                    >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                        </svg>
                        <span class="hidden sm:inline">"Delete"</span>
                    </button>
                </div>
            </header>

            // Delete confirmation dialog
            <Show when=move || show_confirm_dialog.get()>
                <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-sm w-full p-6">
                        <div class="flex items-center gap-3 mb-4">
                            <div class="p-2 bg-red-100 dark:bg-red-900/30 rounded-lg">
                                <svg class="w-5 h-5 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                                </svg>
                            </div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-50">"Delete Document"</h3>
                        </div>
                        <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">"Are you sure? This action cannot be undone."</p>
                        <div class="flex gap-3">
                            <button
                                on:click=move |_| set_show_confirm_dialog.set(false)
                                class="flex-1 px-4 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            >
                                "Cancel"
                            </button>
                            <button
                                on:click=move |_| {
                                    delete_document.dispatch(());
                                    set_show_confirm_dialog.set(false);
                                }
                                class="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition-colors"
                            >
                                "Delete"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Share dialog
            <Show when=move || show_share_dialog.get()>
                <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full p-6">
                        <div class="flex items-center justify-between mb-4">
                            <div class="flex items-center gap-3">
                                <div class="p-2 bg-blue-100 dark:bg-blue-900/30 rounded-lg">
                                    <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"></path>
                                    </svg>
                                </div>
                                <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-50">"Share Document"</h3>
                            </div>
                            <button
                                class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                on:click=move |_| {
                                    set_show_share_dialog.set(false);
                                    set_share_copied.set(false);
                                }
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>

                        <Show
                            when=move || share_token.get().is_some()
                            fallback=move || view! {
                                <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">"Create a share link to allow anyone with the link to view this document (read-only)."</p>
                                <button
                                    class="w-full px-4 py-2.5 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
                                    on:click=move |_| { create_share.dispatch(()); }
                                >
                                    "Create Share Link"
                                </button>
                            }
                        >
                            <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">"Anyone with this link can view the document."</p>
                            <div class="flex gap-2 mb-4">
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 text-xs bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg text-gray-700 dark:text-gray-300 font-mono min-w-0"
                                    prop:value=move || share_url().unwrap_or_default()
                                    readonly
                                />
                                <button
                                    class="shrink-0 px-3 py-2 text-xs font-medium text-blue-700 dark:text-blue-300 bg-blue-50 dark:bg-blue-900/30 rounded-lg hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors"
                                    on:click=move |_| {
                                        if let Some(url) = share_url() {
                                            let js_code = format!("navigator.clipboard.writeText('{}')", url.replace('\'', "\\'"));
                                            let _ = js_sys::eval(&js_code);
                                            set_share_copied.set(true);
                                        }
                                    }
                                >
                                    {move || if share_copied.get() { "Copied!" } else { "Copy" }}
                                </button>
                            </div>
                            <button
                                class="w-full px-4 py-2 text-xs font-medium text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/40 transition-colors"
                                on:click=move |_| { remove_share.dispatch(()); }
                            >
                                "Remove Share Link"
                            </button>
                        </Show>
                    </div>
                </div>
            </Show>

            // Content area
            <div class="flex-1 overflow-hidden">
                <Show
                    when=move || is_editing.get()
                    fallback=move || view! {
                        <div class="h-full overflow-auto p-4 sm:p-6 lg:p-8 prose prose-lg max-w-none dark:prose-invert" inner_html=rendered_html()></div>
                    }
                >
                    // EDIT MODE
                    <div class="flex flex-col h-full sm:flex-col"
                        class=("lg:flex-row", move || show_preview.get())
                    >
                        <Show when=move || show_preview.get()>
                            <div class="h-1/3 sm:h-2/5 lg:h-full lg:w-1/2 border-b lg:border-b-0 lg:border-r border-gray-200 dark:border-gray-700 overflow-hidden">
                                <div
                                    node_ref=preview_ref
                                    class="h-full overflow-auto p-4 sm:p-6 prose prose-lg max-w-none dark:prose-invert"
                                    inner_html=rendered_html()
                                ></div>
                            </div>
                        </Show>
                        <div class="flex-1 min-h-0">
                            <textarea
                                node_ref=editor_ref
                                class="w-full h-full p-4 sm:p-6 border-none outline-none resize-none font-mono text-sm bg-white dark:bg-gray-900 dark:text-gray-50"
                                prop:value=content
                                on:input=move |ev| {
                                    let new_value = event_target_value(&ev);
                                    set_content.set(new_value.clone());
                                    editor_context.0.set(new_value.clone());
                                    dirty.0.set(true);

                                    request_animation_frame(move || {
                                        if let (Some(editor), Some(preview)) = (editor_ref.get_untracked(), preview_ref.get_untracked()) {
                                            let editor_el: &web_sys::HtmlTextAreaElement = &editor;
                                            let preview_el: &web_sys::HtmlElement = &preview;
                                            let cursor_pos = editor_el.selection_start().unwrap_or(Some(0)).unwrap_or(0) as f64;
                                            let content_len = editor_el.value().len() as f64;
                                            if content_len > 0.0 {
                                                let scroll_ratio = cursor_pos / content_len;
                                                let max_preview_scroll = (preview_el.scroll_height() - preview_el.client_height()) as f64;
                                                let target_scroll_top = scroll_ratio * max_preview_scroll;
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
