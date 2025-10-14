use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route, A};
use leptos_router::path;
use leptos_meta::*;
use web_sys::window; // Using your version that compiles

use crate::{
    auth::{AuthProvider, use_auth},
    pages::{DocumentPage, HomePage, LoginPage, RegisterPage},
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

// Define a context type to hold the sidebar signal
#[derive(Clone, Copy)]
pub struct SidebarContext(pub RwSignal<bool>);
pub fn use_sidebar() -> SidebarContext {
    use_context::<SidebarContext>().expect("SidebarContext not found")
}

// Context for the editor content (document body)
#[derive(Clone, Copy)]
pub struct EditorContext(pub RwSignal<String>);
pub fn use_editor() -> EditorContext {
    use_context::<EditorContext>().expect("EditorContext not found")
}

// Context for the LLM chat sidebar visibility
#[derive(Clone, Copy)]
pub struct ChatSidebarContext(pub RwSignal<bool>);
pub fn use_chat_sidebar() -> ChatSidebarContext {
    use_context::<ChatSidebarContext>().expect("ChatSidebarContext not found")
}

// App's only job is to create the providers.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <AuthProvider>
            <AppLayout/>
        </AuthProvider>
    }
}

// AppLayout contains the actual UI and can safely use the contexts.
#[component]
fn AppLayout() -> impl IntoView {
    let auth = use_auth();

    let sidebar_open = RwSignal::new(false);
    provide_context(SidebarContext(sidebar_open));

    let chat_sidebar_open = RwSignal::new(false);
    provide_context(ChatSidebarContext(chat_sidebar_open));

    let editor_open = RwSignal::new(String::new());
    provide_context(EditorContext(editor_open));

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

        <div class="relative flex h-screen bg-gray-100 dark:bg-gray-800">
            <Show when=move || sidebar_open.get()>
                <div
                    class="fixed inset-0 bg-gray-900 bg-opacity-50 z-20 md:hidden"
                    on:click=move |_| sidebar_open.set(false)
                ></div>
            </Show>

            <main class="flex-1 flex flex-col overflow-hidden">
                // persistent header with sidebar toggle for mobile
                <header class="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 p-4 flex items-center md:hidden">
                    <button
                        class="text-gray-500 dark:text-gray-400 focus:outline-none"
                        on:click=move |_| sidebar_open.update(|open| *open = !*open)
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                        </svg>
                    </button>
                    <h1 class="text-xl font-semibold text-gray-800 dark:text-gray-100 ml-4">"Dr. Markdown"</h1>
                </header>            
                // The Router renders the page components (like HomePage)
                <Router base=APP_BASE>
                    <Routes fallback=NotFound>
                        <Route path=path!("/") view=HomePage/>
                        <Route path=path!("/login") view=LoginPage/>
                        <Route path=path!("/register") view=RegisterPage/>
                        <Route path=path!("/documents/:id") view=DocumentPage/>
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
                <h1 class="text-4xl font-bold text-gray-900 mb-4">"404"</h1>
                <p class="text-gray-600 mb-8">"Page not found"</p>
                <A href="/" attr:class="text-blue-600 hover:text-blue-800 underline">
                    "Go home"
                </A>
            </div>
        </div>
    }
}