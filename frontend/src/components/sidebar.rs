use leptos::prelude::*;
use uuid::Uuid;

use crate::models::{Document, DocumentSummary};
use crate::app::use_sidebar;

#[component]
pub fn DocumentSidebar(
    documents: Signal<Vec<DocumentSummary>>,
    selected_document: Signal<Option<Document>>,
    loading: Signal<bool>,
    on_select: impl Fn(Uuid) + Clone + Send + Sync + 'static,
    on_create: impl Fn(String) + Clone + Send + Sync + 'static,
    on_logout: impl Fn() + Clone + Send + Sync + 'static,
    on_theme: impl Fn() + Clone + Send + Sync + 'static,
    user_name: String,
) -> impl IntoView {
    let sidebar = use_sidebar();

    // Create a closure to close the sidebar on mobile after an action
    let close_sidebar = move || sidebar.0.set(false);

    let (new_doc_title, set_new_doc_title) = signal(String::new());
    let (show_create_form, set_show_create_form) = signal(false);

    // --- Create Trigger ---
    let (create_trigger, set_create_trigger) = signal(());
    let on_create_clone = on_create.clone();
    Effect::new(move |prev: Option<()>| {
        create_trigger.get();
        if prev.is_some() {
            let title = new_doc_title.get_untracked().trim().to_string();
            if !title.is_empty() {
                on_create_clone(title);
                set_new_doc_title.set(String::new());
                set_show_create_form.set(false);
            }
        }
        ()
    });

    // Create a trigger signal that will hold the ID of the document to select.
    let (select_trigger, set_select_trigger) = signal(None::<Uuid>);

    // Create an effect that calls the `on_select` prop when the trigger changes.
    let on_select_clone = on_select.clone();
    Effect::new(move |_: Option<()>| {
        if let Some(uuid) = select_trigger.get() {
            on_select_clone(uuid);
        }
        ()
    });

    let close_sidebar = move || sidebar.0.set(false);

    view! {
        <aside class=move || format!(
            "w-80 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 flex flex-col \
            fixed inset-y-0 left-0 z-30 transform {} transition-transform duration-300 ease-in-out \
            md:relative md:translate-x-0",
            if sidebar.0.get() { "translate-x-0" } else { "-translate-x-full" }
        )>
            <header class="p-4 border-b border-gray-200">
                <div class="flex items-center justify-between mb-4">
                     <span class="text-xl font-semibold text-gray-800">"Documents"</span>
                    <button
                        class="text-gray-400 hover:text-gray-600 mr-2"
                        on:click=move |_| {
                            let on_theme = on_theme.clone();
                            on_theme();
                        }
                        title="Toggle Theme"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path d="M12 3v1a9 9 0 100 18v1a10 10 0 110-20z" />
                        </svg>
                    </button>
                    <button
                        class="text-gray-400 hover:text-gray-600"
                        on:click=move |_| {
                            let on_logout = on_logout.clone();
                            on_logout();
                        }
                        title="Logout"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"></path>
                        </svg>
                    </button>
                </div>
                <p class="text-sm text-gray-500 mb-4">"Welcome, " {user_name}</p>
                
                <Show
                    when=move || show_create_form.get()
                    fallback=move || view! {
                        <button
                            class="w-full px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:click=move |_| set_show_create_form.set(true)
                        >
                            "+ New Document"
                        </button>
                    }
                >
                    <div class="space-y-2">
                        <input
                            type="text"
                            placeholder="Document title"
                            class="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=new_doc_title
                            on:input=move |ev| set_new_doc_title.set(event_target_value(&ev))
                            on:keypress=move |ev| {
                                if ev.key() == "Enter" {
                                    set_create_trigger.set(());
                                }
                            }
                        />
                        <div class="flex space-x-2">
                            <button
                                class="flex-1 px-3 py-1 text-xs font-medium text-white bg-blue-600 border border-transparent rounded hover:bg-blue-700"
                                on:click=move |_| set_create_trigger.set(())
                            >
                                "Create"
                            </button>
                            <button
                                class="flex-1 px-3 py-1 text-xs font-medium text-gray-700 bg-gray-100 border border-gray-300 rounded hover:bg-gray-200"
                                on:click=move |_| {
                                    set_show_create_form.set(false);
                                    set_new_doc_title.set(String::new());
                                }
                            >
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </Show>
            </header>

            <div class="flex-1 overflow-y-auto">
                <Show
                    when=move || loading.get()
                    fallback=move || view! {
                        <div class="p-4">
                            <For
                                each=move || documents.get()
                                key=|doc| doc.id
                                children=move |doc| {
                                    let close_sidebar_clone = close_sidebar.clone();
                                    let is_selected = move || {
                                        selected_document.get()
                                            .map(|selected| selected.id == doc.id)
                                            .unwrap_or(false)
                                    };

                                    view! {
                                        <button
                                            class=move || format!(
                                                "w-full text-left p-3 rounded-lg mb-2 transition-colors {}",
                                                if is_selected() {
                                                    "bg-blue-50 border-2 border-blue-200"
                                                } else {
                                                    "hover:bg-gray-50 border-2 border-transparent"
                                                }
                                            )
                                            // capture `set_select_trigger` (which is `Copy`)
                                            // and `doc.id` (which is `Copy`).
                                            on:click=move |_| {
                                                set_select_trigger.set(Some(doc.id));
                                                close_sidebar_clone(); // Close sidebar on selection
                                            }
                                        >
                                            <div class="font-medium text-gray-900 text-sm truncate mb-1">
                                                {doc.title.clone()}
                                            </div>
                                            <div class="text-xs text-gray-500">
                                                {doc.updated_at.format("%b %d, %Y").to_string()}
                                            </div>
                                        </button>
                                    }
                                }
                            />
                            
                            <Show when=move || documents.get().is_empty()>
                                <div class="text-center py-8">
                                    <div class="text-gray-400 mb-2">
                                        <svg class="w-12 h-12 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                                        </svg>
                                    </div>
                                    <p class="text-sm text-gray-500 mb-4">"No documents yet"</p>
                                    <button
                                        class="text-blue-600 hover:text-blue-700 text-sm font-medium"
                                        on:click=move |_| set_show_create_form.set(true)
                                    >
                                        "Create your first document"
                                    </button>
                                </div>
                            </Show>
                        </div>
                    }
                >
                    <div class="p-4 text-center">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
                        <p class="text-sm text-gray-500 mt-2">"Loading documents..."</p>
                    </div>
                </Show>
            </div>
        </aside>
    }
}