use std::{fmt::Display, str::FromStr};

use dioxus::prelude::*;
use crate::components::input::Input;

#[derive(PartialEq, Clone)]
pub enum InputType<T: PartialOrd + Clone + FromStr + 'static> {
    Text,
    Password,
    Number {
        min: Option<T>,
        max: Option<T>
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ConstrainedInputProps<T: PartialOrd + Clone + FromStr + 'static> {
    pub id: String,
    pub placeholder: Option<String>,
    pub value: String,
    pub onchange: Callback<T, ()>,
    pub r#type: InputType<T>
}

fn meets_criteria<T>(input_type: InputType<T>, value: String) -> Option<T> where T: PartialOrd + Clone + FromStr + 'static {
    match input_type {
        InputType::Text | InputType::Password => if let Ok(parsed) = value.parse::<T>() {
                Some(parsed)
            } else {
                None
            },
        InputType::Number { min, max } => {
            if let Ok(parsed) = value.parse::<T>() {
                if let Some(x) = min && x > parsed { return None };
                if let Some(x) = max && x < parsed { return None };
                Some(parsed)
            } else {
                None
            }
        }
    }
}

#[component]
pub fn ConstrainedInput<T: PartialOrd + Clone + FromStr + Display + dioxus::dioxus_core::IntoAttributeValue + 'static>(props: ConstrainedInputProps<T>) -> Element {

    let mut error = use_signal(|| None);

    let mut notify_constrain_violation = move |input_type: InputType<T>| {
        let description = match input_type {
            InputType::Text | InputType::Password => "".to_string(),
            InputType::Number { min, max } => {
                let mut base = String::from("The input must be a number ");
                if min.is_some() && max.is_some() {
                    base.push_str(&format!("inclusively between {} and {}.", min.unwrap(), max.unwrap()));
                    base
                } else if let Some(min_v) = min {
                    base.push_str(&format!("above or equal {min_v}."));
                    base
                } else if let Some(max_v) = max {
                    base.push_str(&format!("bellow or equal {max_v}."));
                    base
                } else {
                    "".to_string()
                }
            }
        };

        error.set(Some(description));
    };

    rsx! {
        span { class: "flex flex-col grow",
            Input {
                placeholder: props.placeholder,
                r#type: {
                    match props.r#type {
                        InputType::Text => "text",
                        InputType::Password => "password",
                        InputType::Number { max: _, min: _ } => "number",
                    }
                },

                min: match props.r#type {
                    InputType::Number { ref min, max: _ } => Some(min),
                    _ => None,
                },
                max: match props.r#type {
                    InputType::Number { min: _, ref max } => Some(max),
                    _ => None,
                },

                value: props.value,
                onchange: move |evt: Event<FormData>| {
                    let value = evt.value();
                    let input_type = props.r#type.clone();
                    if let Some(parsed) = meets_criteria(input_type.clone(), value.clone()) {
                        props.onchange.call(parsed);
                    } else {
                        notify_constrain_violation(input_type);
                    }
                },
            }
            if let Some(description) = error() {
                small { class: "text-red-500 font-medium block", {description} }
            }
        }
    }
}
