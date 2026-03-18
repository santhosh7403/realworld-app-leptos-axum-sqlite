use leptos::{html::Input, prelude::*};
use leptos_meta::*;
use leptos_router::components::*;

use crate::auth::{SignupAction, SignupResponse};

#[component]
pub fn SignupForm(signup: ServerAction<SignupAction>) -> impl IntoView {
    let (signup_status, set_signup_status) = signal(String::new());
    let result_of_call = signup.value();
    let show_modal: RwSignal<bool> = use_context().expect("show_modal context should be available");
    show_modal.set(true);

    Effect::new(move || {
        signup.value().get();
        result_of_call.with(|msg| {
            msg.as_ref().map(|inner| match inner {
                Ok(SignupResponse::ValidationError(x)) => {
                    set_signup_status.set(format!("Problem while validating: {x}."));
                }
                Ok(SignupResponse::CreateUserError(x)) => {
                    set_signup_status.set(format!("Problem while creating user: {x}."));
                }
                Ok(SignupResponse::Success) => {
                    set_signup_status.set("Signup Successful.".to_string());
                }
                Err(x) => {
                    tracing::error!("Problem during signup: {x:?}");
                    set_signup_status
                        .set("There was some problem with signup, try again later".to_string());
                }
            })
        });
    });

    let on_signup_event = move |sa: SignupAction| {
        signup.dispatch(sa);
    };

    let navigate_home = || {
        let navigate = leptos_router::hooks::use_navigate();
        navigate("/", Default::default());
    };

    let on_cancel_signup_event = move || {
        signup.clear();
        show_modal.set(false);
        navigate_home();
    };

    view! {
        <Show when=move || show_modal.get()>
            <SignupModal on_in=on_signup_event on_cancel=on_cancel_signup_event signup_status />
        </Show>
    }
}

#[component]
fn SignupModal<A, C>(on_in: A, on_cancel: C, signup_status: ReadSignal<String>) -> impl IntoView
where
    A: Fn(SignupAction) + 'static + Send,
    C: Fn() + 'static + Send + Copy,
{
    let acc_user: NodeRef<Input> = NodeRef::new();
    let acc_email: NodeRef<Input> = NodeRef::new();
    let acc_password: NodeRef<Input> = NodeRef::new();

    let on_in_event = move |_| {
        if signup_status.get().starts_with("Signup Successful") {
            on_cancel()
        } else {
            let user = acc_user.get().expect("user <input> to exist").value();
            let email = acc_email.get().expect("email <input> to exist").value();
            let password = acc_password
                .get()
                .expect("password <input> to exist")
                .value();

            on_in(SignupAction {
                username: user,
                email,
                password,
            })
        }
    };

    let (passwd_visible, set_passwd_visible) = signal(false);
    let create_button_string = move || {
        format!(
            "{}",
            if signup_status.get().starts_with("Signup Successful.") {
                "Back to Home"
            } else {
                "Create Account"
            }
        )
    };

    view! {
        <Title text="Signup" />
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60">
            <div class="block rounded-lg bg-white dark:bg-gray-800 dark:text-gray-100 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70">

                <h5 class="mb-5 text-xl font-medium leading-tight text-neutral-500 dark:text-neutral-400">
                    Create an account.
                </h5>
                <p class="text-xs-center py-6">
                    <span class="text-blue-500 font-medium">
                        <A href="/login">"Have an account already? Click here to login "</A>
                    </span>
                </p>
                <form>
                    <label class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2" for="username">
                        User Name
                    </label>
                    <div class="mb-5">
                        <input
                            node_ref=acc_user
                            class="input-field-common"
                            id="username"
                            name="username"
                            type="text"
                            value=move || { String::new() }
                            placeholder="username"
                            required=true
                        />
                    </div>
                    <label class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2" for="email">
                        Email
                    </label>
                    <div class="mb-5">
                        <input
                            node_ref=acc_email
                            class="input-field-common"
                            id="email"
                            name="email"
                            type="email"
                            value=move || { String::new() }
                            placeholder="Email"
                            required=true
                        />
                    </div>
                    <label class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2" for="password">
                        Password
                    </label>
                    <div class="mb-5 relative">
                        <input
                            node_ref=acc_password
                            class="input-field-common"
                            id="password"
                            name="password"
                            type=move || {
                                format!(
                                    "{}",
                                    if passwd_visible.get() { "text" } else { "password" },
                                )
                            }
                            value=move || { String::new() }
                            placeholder="Password"
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
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg"
                            on:click=on_in_event
                        >
                            {create_button_string}
                        </button>
                        <button
                            type="cancel"
                            class="bg-gray-300 hover:bg-gray-400 px-5 py-3 text-white rounded-lg"
                            on:click=move |_| on_cancel()
                        >
                            Cancel
                        </button>
                    </div>
                    <div>
                        <span class=move || {
                            format!(
                                "font-medium {}",
                                if signup_status.get().starts_with("Signup Successful.") {
                                    "text-green-500"
                                } else {
                                    "text-red-500"
                                },
                            )
                        }>{move || signup_status.get()}</span>
                    </div>

                </form>
            </div>
        </div>
    }
}
