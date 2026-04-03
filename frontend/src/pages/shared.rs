use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

use crate::api::ApiClient;
use crate::models::SharedDocument;
use crate::pages::home::render_markdown;

#[component]
pub fn SharedDocumentPage() -> impl IntoView {
    let params = use_params_map();

    let (document, set_document) = signal(None::<SharedDocument>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        let token = params.with(|p| p.get("token").map(|t| t.to_string()));

        set_loading.set(true);
        set_error.set(None);

        if let Some(token) = token {
            spawn_local(async move {
                let client = ApiClient::new();
                match client.get_shared_document(&token).await {
                    Ok(doc) => {
                        set_document.set(Some(doc));
                        set_loading.set(false);
                    }
                    Err(err) => {
                        set_error.set(Some(err.error));
                        set_loading.set(false);
                    }
                }
            });
        } else {
            set_error.set(Some("Invalid share link".to_string()));
            set_loading.set(false);
        }
    });

    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <Show
                when=move || loading.get()
                fallback=move || view! {
                    <Show
                        when=move || document.get().is_some()
                        fallback=move || view! {
                            <div class="flex items-center justify-center min-h-screen">
                                <div class="text-center p-8">
                                    <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                                    </svg>
                                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-50 mb-2">"Document not found"</h1>
                                    <p class="text-gray-500 dark:text-gray-400">"This share link may have expired or been removed."</p>
                                    {move || error.get().map(|err| view! {
                                        <p class="text-red-500 mt-2 text-sm">{err}</p>
                                    })}
                                </div>
                            </div>
                        }
                    >
                        {move || document.get().map(|doc| {
                            let rendered = render_markdown(&doc.content);
                            view! {
                                <div class="max-w-4xl mx-auto">
                                    <header class="sticky top-0 z-10 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-b border-gray-200 dark:border-gray-700 px-4 sm:px-6 lg:px-8 py-4">
                                        <div class="flex items-center gap-3">
                                            <svg class="w-6 h-6 text-blue-600 dark:text-blue-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                                            </svg>
                                            <h1 class="text-xl font-semibold text-gray-900 dark:text-gray-50 truncate">{doc.title.clone()}</h1>
                                            <span class="ml-auto text-xs text-gray-400 dark:text-gray-500 shrink-0 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded">"Shared (read-only)"</span>
                                        </div>
                                    </header>
                                    <article class="px-4 sm:px-6 lg:px-8 py-8 prose prose-lg max-w-none dark:prose-invert" inner_html=rendered></article>
                                </div>
                            }
                        })}
                    </Show>
                }
            >
                <div class="flex items-center justify-center min-h-screen">
                    <div class="text-center">
                        <div class="animate-spin rounded-full h-10 w-10 border-b-2 border-blue-600 mx-auto mb-4"></div>
                        <p class="text-gray-500 dark:text-gray-400">"Loading shared document..."</p>
                    </div>
                </div>
            </Show>
        </div>
    }
}
