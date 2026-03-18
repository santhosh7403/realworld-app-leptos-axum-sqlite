use crate::auth::LoginAction;
use leptos::{html::Input, prelude::*};
use leptos_meta::*;

#[component]
pub fn LoginForm(login: ServerAction<LoginAction>) -> impl IntoView {
    login.clear();
    let (login_status, set_login_status) = signal(String::new());
    let result_of_call = login.value();
    let show_modal: RwSignal<bool> = use_context().expect("show_modal context should be available");
    show_modal.set(true);

    Effect::new(move || {
        login.version().get();
        result_of_call.with(|msg| {
            msg.as_ref().map(|inner| match inner {
                Ok(_) => {
                    leptos::logging::log!("Login Successful.");
                    tracing::debug!("Login Successful");
                    set_login_status.set("Login Successful.".to_string());
                    show_modal.set(false);
                }
                Err(x) => match x {
                    ServerFnError::ServerError(err) if err.starts_with("Unsuccessful") => {
                        tracing::debug!("Login failed. Incorrect User or Password: {}", err);
                        set_login_status
                            .set("Login failed. Incorrect User or Password".to_string());
                    }
                    _ => {
                        tracing::debug!("There was some problem with login: {}", x);
                        set_login_status
                            .set("Login failed. Incorrect User or Password.".to_string());
                    }
                },
            })
        });
    });

    let on_signin_event = move |la: LoginAction| {
        login.dispatch(la);
    };

    let navigate_home = || {
        let navigate = leptos_router::hooks::use_navigate();
        navigate("/", Default::default());
    };

    let on_cancel_signin_event = move || {
        login.clear();
        show_modal.set(false);
        navigate_home();
    };

    view! {
        <Show when=move || show_modal.get()>
            <LoginModal on_in=on_signin_event on_cancel=on_cancel_signin_event login_status />
        </Show>
    }
}

#[component]
fn LoginModal<A, C>(on_in: A, on_cancel: C, login_status: ReadSignal<String>) -> impl IntoView
where
    A: Fn(LoginAction) + 'static + Send,
    C: Fn() + 'static + Send,
{
    let login_user: NodeRef<Input> = NodeRef::new();
    let login_pass: NodeRef<Input> = NodeRef::new();

    let on_in_event = move |_| {
        let user = login_user.get().expect("<input> to exist").value();
        let pass = login_pass.get().expect("<input> to exist").value();

        on_in(LoginAction {
            username: user,
            password: pass,
        })
    };

    let (passwd_visible, set_passwd_visible) = signal(false);

    view! {
        <Title text="Login" />
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60">
            <div class="block rounded-lg bg-white dark:bg-gray-800 dark:text-gray-100 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70">
                <h5 class="mb-5 text-xl font-medium leading-tight text-neutral-500 dark:text-neutral-400">
                    Please login with your credentials.
                </h5>
                <form>
                    <label class="block text-gray-700 dark:text-gray-300 text-sm font-bold " for="username">
                        User Name
                    </label>
                    <div class="mb-5">
                        <input
                            node_ref=login_user
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 dark:text-gray-300 leading-tight focus:outline-none focus:shadow-outline"
                            id="username"
                            name="username"
                            type="text"
                            value=move || { String::new() }
                            placeholder="username"
                            required=true
                        />
                    </div>
                    <label class="block text-gray-700 dark:text-gray-300 text-sm font-bold" for="password">
                        Password
                    </label>
                    <div class="mb-5 relative">
                        <input
                            node_ref=login_pass
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 dark:text-gray-300 leading-tight focus:outline-none focus:shadow-outline"
                            id="password"
                            name="password"
                            type=move || {
                                format!(
                                    "{}",
                                    if passwd_visible.get() { "text" } else { "password" },
                                )
                            }
                            value=move || { String::new() }
                            placeholder="password"
                            required=true
                        />
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer"
                            on:click=move |_| {
                                set_passwd_visible.update(|val| *val = !*val);
                            }
                        >
                            <i class=move || {
                                format!(
                                    "{}",
                                    if passwd_visible.get() {
                                        "far fa-eye"
                                    } else {
                                        "far fa-eye-slash"
                                    },
                                )
                            }></i>
                        </span>
                    </div>
                    <div class="flex flex-row-reverse space-x-4 space-x-reverse">
                        <button
                            type="button"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-2 text-white rounded-lg"
                            on:click=on_in_event
                        >
                            Signin
                        </button>
                        <button
                            type="cancel"
                            class="bg-gray-300 hover:bg-gray-400 px-5 py-2 text-white rounded-lg"
                            on:click=move |_| on_cancel()
                        >
                            Cancel
                        </button>
                        <a class="p-2 hover:text-blue-500 hover:underline" href="/reset_password">
                            Forgot password?
                        </a>
                    </div>
                    <div>
                        <span class=move || {
                            format!(
                                "{}",
                                if login_status.get() == "Login Successful." {
                                    "text-green-500 font-medium"
                                } else {
                                    "text-red-500 font-medium"
                                },
                            )
                        }>{move || login_status.get()}</span>
                    </div>
                </form>

            </div>
        </div>
    }
}
