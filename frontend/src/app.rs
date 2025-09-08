use leptos::prelude::*;
use leptos_router::components::{Router, Routes, Route, A};
use leptos_router::path;

use crate::{
    auth::AuthProvider,
    pages::{DocumentPage, HomePage, LoginPage, RegisterPage},
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <AuthProvider>
            <Router>
                <main class="min-h-screen bg-gray-50">
                    <Routes fallback=NotFound>
                        <Route path=path!("/") view=HomePage/>
                        <Route path=path!("/login") view=LoginPage/>
                        <Route path=path!("/register") view=RegisterPage/>
                        <Route path=path!("/documents/:id") view=DocumentPage/>
                    </Routes>
                </main>
            </Router>
        </AuthProvider>
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