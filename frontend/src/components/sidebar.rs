use leptos::prelude::*;
use uuid::Uuid;

use crate::models::{Document, DocumentSummary};
use crate::app::{use_sidebar, use_dirty};

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
    let dirty = use_dirty();

    let (new_doc_title, set_new_doc_title) = signal(String::new());
    let (show_create_form, set_show_create_form) = signal(false);

    // Unsaved changes confirmation
    let (pending_select_id, set_pending_select_id) = signal(None::<Uuid>);
    let (show_unsaved_dialog, set_show_unsaved_dialog) = signal(false);

    // Create trigger
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

    // Select trigger — only fires after unsaved check passes
    let (select_trigger, set_select_trigger) = signal(None::<Uuid>);
    let on_select_clone = on_select.clone();
    Effect::new(move |_: Option<()>| {
        if let Some(uuid) = select_trigger.get() {
            on_select_clone(uuid);
        }
        ()
    });

    let close_sidebar = move || sidebar.0.set(false);

    // Called when user clicks a document in the list
    let try_select = move |doc_id: Uuid| {
        let is_same = selected_document.get()
            .map(|sel| sel.id == doc_id)
            .unwrap_or(false);
        if is_same {
            return;
        }
        if dirty.0.get() {
            set_pending_select_id.set(Some(doc_id));
            set_show_unsaved_dialog.set(true);
        } else {
            set_select_trigger.set(Some(doc_id));
            close_sidebar();
        }
    };

    view! {
        <aside class=move || format!(
            "w-72 lg:w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col \
            fixed inset-y-0 left-0 z-30 transform {} transition-transform duration-300 ease-in-out \
            md:relative md:translate-x-0",
            if sidebar.0.get() { "translate-x-0" } else { "-translate-x-full" }
        )>
            // Header
            <header class="p-4 border-b border-gray-200 dark:border-gray-700 shrink-0">
                <div class="flex items-center justify-between mb-3">
                    <div class="flex items-center gap-2">
                        <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                        </svg>
                        <span class="text-lg font-semibold text-gray-800 dark:text-gray-100">"Documents"</span>
                    </div>
                    <div class="flex items-center gap-1">
                        <button
                            class="p-2 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                            on:click=move |_| {
                                let on_theme = on_theme.clone();
                                on_theme();
                            }
                            title="Toggle Theme"
                        >
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="5"/>
                                <line x1="1" y1="12" x2="3" y2="12"/>
                                <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
                                <line x1="12" y1="1" x2="12" y2="3"/>
                                <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
                                <line x1="12" y1="21" x2="12" y2="23"/>
                            </svg>
                        </button>
                        <button
                            class="p-2 rounded-lg text-gray-400 hover:text-red-500 dark:hover:text-red-400 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
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
                        // Close button on mobile
                        <button
                            class="p-2 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors md:hidden"
                            on:click=move |_| sidebar.0.set(false)
                            title="Close sidebar"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>
                </div>
                <p class="text-xs text-gray-500 dark:text-gray-400 mb-3">"Welcome, " {user_name}</p>

                <Show
                    when=move || show_create_form.get()
                    fallback=move || view! {
                        <button
                            class="w-full flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 transition-colors"
                            on:click=move |_| set_show_create_form.set(true)
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
                            </svg>
                            "New Document"
                        </button>
                    }
                >
                    <div class="space-y-2">
                        <input
                            type="text"
                            placeholder="Document title"
                            class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=new_doc_title
                            on:input=move |ev| set_new_doc_title.set(event_target_value(&ev))
                            on:keypress=move |ev| {
                                if ev.key() == "Enter" {
                                    set_create_trigger.set(());
                                }
                            }
                        />
                        <div class="flex gap-2">
                            <button
                                class="flex-1 px-3 py-1.5 text-xs font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
                                on:click=move |_| set_create_trigger.set(())
                            >
                                "Create"
                            </button>
                            <button
                                class="flex-1 px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
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

            // Document list
            <div class="flex-1 overflow-y-auto">
                <Show
                    when=move || loading.get()
                    fallback=move || view! {
                        <div class="p-3">
                            <For
                                each=move || documents.get()
                                key=|doc| (doc.id, doc.title.clone())
                                children=move |doc| {
                                    let try_select = try_select.clone();
                                    let is_selected = move || {
                                        selected_document.get()
                                            .map(|selected| selected.id == doc.id)
                                            .unwrap_or(false)
                                    };

                                    view! {
                                        <button
                                            class=move || format!(
                                                "w-full text-left px-3 py-2.5 rounded-lg mb-1 transition-all {}",
                                                if is_selected() {
                                                    "bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-700"
                                                } else {
                                                    "hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent"
                                                }
                                            )
                                            on:click=move |_| {
                                                try_select(doc.id);
                                            }
                                        >
                                            <div class="font-medium text-gray-900 dark:text-gray-100 text-sm truncate">
                                                {doc.title.clone()}
                                            </div>
                                            <div class="text-xs text-gray-400 dark:text-gray-500 mt-0.5">
                                                {doc.updated_at.format("%b %d, %Y").to_string()}
                                            </div>
                                        </button>
                                    }
                                }
                            />

                            <Show when=move || documents.get().is_empty()>
                                <div class="text-center py-12 px-4">
                                    <svg class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                                    </svg>
                                    <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">"No documents yet"</p>
                                    <button
                                        class="text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 text-sm font-medium"
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
                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">"Loading documents..."</p>
                    </div>
                </Show>
            </div>
        </aside>

        // Unsaved changes dialog
        <Show when=move || show_unsaved_dialog.get()>
            <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-sm w-full p-6">
                    <div class="flex items-center gap-3 mb-4">
                        <div class="p-2 bg-amber-100 dark:bg-amber-900/30 rounded-lg">
                            <svg class="w-5 h-5 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
                            </svg>
                        </div>
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-50">"Unsaved Changes"</h3>
                    </div>
                    <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
                        "You have unsaved changes. Do you want to discard them and switch documents?"
                    </p>
                    <div class="flex gap-3">
                        <button
                            class="flex-1 px-4 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            on:click=move |_| {
                                set_show_unsaved_dialog.set(false);
                                set_pending_select_id.set(None);
                            }
                        >
                            "Cancel"
                        </button>
                        <button
                            class="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-amber-600 rounded-lg hover:bg-amber-700 transition-colors"
                            on:click=move |_| {
                                dirty.0.set(false);
                                if let Some(id) = pending_select_id.get_untracked() {
                                    set_select_trigger.set(Some(id));
                                    close_sidebar();
                                }
                                set_show_unsaved_dialog.set(false);
                                set_pending_select_id.set(None);
                            }
                        >
                            "Discard & Switch"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
