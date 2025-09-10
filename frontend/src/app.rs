use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route, A};
use leptos_router::path;
use leptos_meta::*;
use web_sys::window; // Using your version that compiles

use crate::{
    auth::{AuthProvider, use_auth},
    pages::{DocumentPage, HomePage, LoginPage, RegisterPage},
};

pub const APP_BASE: &str = match option_env!("LEPTOS_APP_BASE_PATH") {
    Some(path) => path,
    None => "",
};

pub const THEME_LIGHT: &str = "light";
pub const THEME_DARK: &str = "dark";

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
    // This is now SAFE because AppLayout is a child of AuthProvider.
    let auth = use_auth();

    // This Effect now has access to `auth` and will work correctly.
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
        <Router base=APP_BASE>
            <main class="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-800 dark:text-gray-200">
                <Routes fallback=NotFound>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/login") view=LoginPage/>
                    <Route path=path!("/register") view=RegisterPage/>
                    <Route path=path!("/documents/:id") view=DocumentPage/>
                </Routes>
            </main>
        </Router>
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