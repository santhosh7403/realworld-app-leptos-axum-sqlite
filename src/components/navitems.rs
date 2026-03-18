use crate::auth::*;
use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub(crate) fn NavItems(
    login: ServerAction<LoginAction>,
    logout: ServerAction<LogoutAction>,
    username: UsernameSignal,
) -> impl IntoView {
    let profile_label = move || username.get().unwrap_or_default();
    let profile_href = move || format!("/profile/{}", profile_label());

    let navigate_login = move |_| {
        let navigate = leptos_router::hooks::use_navigate();
        navigate("/login", Default::default());
    };

    let theme_mode = use_context::<RwSignal<String>>().expect("theme_mode should be provided");

    let toggle_theme = move |_| {
        let current_theme = theme_mode.get();
        let new_theme = if current_theme == "dark" {
            "light".to_string()
        } else {
            "dark".to_string()
        };
        theme_mode.set(new_theme.clone());
        if username.get().is_some() {
            leptos::task::spawn(async move {
                let _ = crate::auth::update_theme_mode(new_theme.clone()).await;
            });
        }
    };

    view! {
        <div class="bg-gray-800 text-white shadow-lg md:relative md:top-0 md:left-0 md:right-auto md:w-full
        rounded-b-xl px-4 py-3 md:py-4 flex justify-between items-center">
            <div class="flex justify-around items-center flex-1">
                <A href="/">
                    <div class="group navitem">
                        <i class="fas fa-home navitem-icon"></i>
                        <span class="text-xs md:text-base mt-1 font-semibold">Home</span>
                    </div>
                </A>
                <Show
                    when=move || username.with(Option::is_none)
                    fallback=move || {
                        view! {
                            <A href="/editor">
                                <div class="group navitem">
                                    <i class="fa-solid fa-square-plus navitem-icon"></i>
                                    <span class="text-xs md:text-base mt-1 font-semibold">
                                        New Article
                                    </span>
                                </div>
                            </A>

                            <A href="/settings">
                                <div class="group navitem">
                                    <i class="fa-solid fa-gear navitem-icon"></i>
                                    <span class="text-xs md:text-base mt-1 font-semibold">
                                        Settings
                                    </span>
                                </div>
                            </A>
                            <A href=profile_href.clone()>
                                <div class="group navitem">
                                    <i class="fa-regular fa-circle-user navitem-icon"></i>
                                    <span class="text-xs md:text-base mt-1 font-semibold">
                                        {profile_label}
                                    </span>
                                </div>
                            </A>

                            <ActionForm action=logout>
                                <button
                                    class="items-center border-none bg-transparent
                                    focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                                    on:click=move |_| login.clear()
                                >
                                    <div class="group navitem">
                                        <i class="fa-solid fa-right-from-bracket navitem-icon"></i>
                                        <span class="text-xs md:text-base mt-1 font-semibold">
                                            Logout
                                        </span>
                                    </div>
                                </button>
                            </ActionForm>
                        }
                    }
                >
                    <A href="/signup">
                        <div class="group navitem">
                            <i class="fa-solid fa-user-plus navitem-icon"></i>
                            <span class="text-xs md:text-base mt-1 font-semibold">Sign up</span>
                        </div>
                    </A>

                    <button
                        on:click=navigate_login
                        class="focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                    >
                        <div class="group navitem">
                            <i class="fa-solid fa-right-to-bracket navitem-icon"></i>
                            <span class="text-xs md:text-base mt-1 font-semibold">Login</span>
                        </div>
                    </button>
                </Show>
            </div>
            <div class="flex items-center border-l-2 border-dotted border-gray-600 pl-4 ml-2">
                <button
                    on:click=toggle_theme
                    class="rounded-full p-2"
                    // class="focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-full p-2"
                    aria-label="Toggle Dark Mode"
                >
                    <div class="flex items-center justify-center group navitem">
                        {move || if theme_mode.get() == "dark" {
                            view! {
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 md:w-8 md:h-8 transition-transform hover:scale-110 text-gray-300 hover:text-yellow-400">
                                  <path stroke-linecap="round" stroke-linejoin="round" d="M21.752 15.002A9.72 9.72 0 0 1 18 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 0 0 3 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 0 0 9.002-5.998Z" />
                                </svg>
                            }.into_any()
                        } else {
                            view! {
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 md:w-8 md:h-8 transition-transform hover:scale-110 text-gray-300 hover:text-yellow-400">
                                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v2.25m6.364.386-1.591 1.591M21 12h-2.25m-.386 6.364-1.591-1.591M12 18.75V21m-4.773-4.227-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0Z" />
                                </svg>
                            }.into_any()
                        }}
                    </div>
                </button>
            </div>
        </div>
    }
}
