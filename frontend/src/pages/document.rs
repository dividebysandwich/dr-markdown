use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use uuid::Uuid;
use std::sync::Arc;
use leptos::task::spawn_local;

use crate::{api::ApiClient, auth::use_auth, models::Document};
use crate::app::APP_BASE;

#[component]
pub fn DocumentPage() -> impl IntoView {
    let auth = use_auth();
    let params = use_params_map();
    let navigate = use_navigate();
    let navigate_for_delete = navigate.clone();
    let navigate_for_auth = navigate.clone();
    
    let (delete_trigger, set_delete_trigger) = signal(());

    Effect::new(move |prev: Option<()>| {
        // Establish a dependency on the trigger signal.
        delete_trigger.get();

        // The `prev` argument is `None` on the first run of the effect.
        // On subsequent runs (i.e., when `set_delete_trigger` is called),
        // `prev` will be `Some(())`. This is how we prevent navigating on load.
        if prev.is_some() {
            navigate_for_delete("/", Default::default());
        }
        
        // The return value of the effect becomes the `prev` for the next run.
        ()
    });

    Effect::new(move |_| {
        if !auth.state.get().loading && auth.state.get().user.is_none() {
            navigate_for_auth("/login", Default::default());
        }
    });

    let (document_and_client, set_document_and_client) = signal(None::<(Arc<ApiClient>, Document)>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    Effect::new(move |_| {
        let auth_token = auth.state.get().token.clone();
        let doc_id_str = params.with(|p| p.get("id").map(|id| id.to_string()));

        set_loading.set(true);
        set_error.set(None);
        set_document_and_client.set(None);

        if let (Some(token), Some(doc_id_str)) = (auth_token, doc_id_str) {
            if let Ok(doc_id) = Uuid::parse_str(&doc_id_str) {
                spawn_local(async move {
                    let client = ApiClient::with_token(token);
                    let client_arc = Arc::new(client);
                    match client_arc.get_document(doc_id).await {
                        Ok(doc) => {
                            set_document_and_client.set(Some((client_arc, doc)));
                            set_loading.set(false);
                        }
                        Err(err) => {
                            set_error.set(Some(err.error));
                            set_loading.set(false);
                        }
                    }
                });
            } else {
                set_error.set(Some("Invalid document ID".to_string()));
                set_loading.set(false);
            }
        } else {
            set_loading.set(false);
        }
    });

    // Add sidebar_open signal
    let (sidebar_open, set_sidebar_open) = signal(false);

    view! {
        <Show when=move || auth.state.get().loading>
            <div class="flex items-center justify-center h-full">
                <div class="text-lg">"Loading..."</div>
            </div>
        </Show>

        <Show
            when=move || !auth.state.get().loading && loading.get()
            fallback=move || view! {
                <Show
                    when=move || document_and_client.get().is_some()
                    fallback=move || view! {
                        <div class="flex items-center justify-center h-full">
                            <div class="text-center">
                                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-50 mb-4">"Document not found"</h1>
                                {move || error.get().map(|err| view! {
                                    <p class="text-red-600 mb-4">{err}</p>
                                })}
                                <A href=format!("{}/", APP_BASE) prop:class="text-blue-600 hover:text-blue-800 underline">
                                    "Go back home"
                                </A>
                            </div>
                        </div>
                    }
                >
                    {move || document_and_client.get().map(|(client, doc)| {
                        view! {
                            <crate::pages::home::DocumentEditor
                                document={doc.clone()}
                                on_save=move |updated_doc| {
                                    set_document_and_client.update(|current| {
                                        if let Some((_, ref mut current_doc)) = current {
                                            *current_doc = updated_doc;
                                        }
                                    });
                                }
                                on_delete=move |_| {
                                    set_delete_trigger.set(());
                                }
                                client={client.clone()}
                            />
                        }
                    })}
                </Show>
            }
        >
            <div class="flex items-center justify-center h-full">
                <div class="text-lg">"Loading document..."</div>
            </div>
        </Show>
    }
}