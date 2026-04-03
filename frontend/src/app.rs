use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route, A};
use leptos_router::path;
use leptos_meta::*;
use web_sys::window;

use crate::{
    auth::{AuthProvider, use_auth},
    pages::{DocumentPage, HomePage, LoginPage, RegisterPage, SharedDocumentPage},
    components::ChatSidebar
};

pub const APP_BASE: &str = match option_env!("LEPTOS_APP_BASE_PATH") {
    Some(path) => path,
    None => "",
};

pub const KROKI_URL: &str = match option_env!("KROKI_URL") {
    Some(url) => url,
    None => "https://kroki.io",
};

pub const THEME_LIGHT: &str = "light";
pub const THEME_DARK: &str = "dark";

#[derive(Clone, Copy)]
pub struct SidebarContext(pub RwSignal<bool>);
pub fn use_sidebar() -> SidebarContext {
    use_context::<SidebarContext>().expect("SidebarContext not found")
}

#[derive(Clone, Copy)]
pub struct EditorContext(pub RwSignal<String>);
pub fn use_editor() -> EditorContext {
    use_context::<EditorContext>().expect("EditorContext not found")
}

#[derive(Clone, Copy)]
pub struct ChatSidebarContext(pub RwSignal<bool>);
pub fn use_chat_sidebar() -> ChatSidebarContext {
    use_context::<ChatSidebarContext>().expect("ChatSidebarContext not found")
}

/// Tracks whether the current document has unsaved edits.
#[derive(Clone, Copy)]
pub struct DirtyContext(pub RwSignal<bool>);
pub fn use_dirty() -> DirtyContext {
    use_context::<DirtyContext>().expect("DirtyContext not found")
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <AuthProvider>
            <AppLayout/>
        </AuthProvider>
    }
}

#[component]
fn AppLayout() -> impl IntoView {
    let auth = use_auth();

    let sidebar_open = RwSignal::new(false);
    provide_context(SidebarContext(sidebar_open));

    let chat_sidebar_open = RwSignal::new(false);
    provide_context(ChatSidebarContext(chat_sidebar_open));

    let editor_open = RwSignal::new(String::new());
    provide_context(EditorContext(editor_open));

    let dirty = RwSignal::new(false);
    provide_context(DirtyContext(dirty));

    Effect::new(move |_| {
        let theme = auth.state.get().user
            .as_ref()
            .map(|u| u.theme.clone())
            .unwrap_or_else(|| "light".to_string());

        let doc = window().expect("should have a Window").document().expect("should have a Document");
        let html_element = doc.document_element().expect("should have <html> element");

        if theme == "dark" {
            let _ = html_element.class_list().add_1("dark");
        } else {
            let _ = html_element.class_list().remove_1("dark");
        }
    });

    view! {
        <div class="relative flex h-screen bg-gray-50 dark:bg-gray-900">
            <Show when=move || sidebar_open.get()>
                <div
                    class="fixed inset-0 bg-black/40 backdrop-blur-sm z-20 md:hidden"
                    on:click=move |_| sidebar_open.set(false)
                ></div>
            </Show>

            <main class="flex-1 flex flex-col overflow-hidden">
                <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3 flex items-center gap-3 md:hidden">
                    <button
                        class="p-2 rounded-lg text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                        on:click=move |_| sidebar_open.update(|open| *open = !*open)
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                        </svg>
                    </button>
                    <h1 class="text-lg font-semibold text-gray-800 dark:text-gray-100">"Dr. Markdown"</h1>
                </header>
                <Router base=APP_BASE>
                    <Routes fallback=NotFound>
                        <Route path=path!("/") view=HomePage/>
                        <Route path=path!("/login") view=LoginPage/>
                        <Route path=path!("/register") view=RegisterPage/>
                        <Route path=path!("/documents/:id") view=DocumentPage/>
                        <Route path=path!("/shared/:token") view=SharedDocumentPage/>
                    </Routes>
                </Router>
            </main>
        </div>
        <ChatSidebar />
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <div class="text-center">
                <h1 class="text-4xl font-bold text-gray-900 dark:text-gray-50 mb-4">"404"</h1>
                <p class="text-gray-600 dark:text-gray-400 mb-8">"Page not found"</p>
                <A href="/" attr:class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 underline">
                    "Go home"
                </A>
            </div>
        </div>
    }
}
