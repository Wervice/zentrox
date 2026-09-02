use dioxus::prelude::*;
use plotters::{prelude::*, style::full_palette::{GREY_200, GREY_500, GREY_600, GREY_800}};
use plotters_canvas::CanvasBackend;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, wasm_bindgen::{JsCast, JsValue}};

#[derive(Clone, Debug)]
struct RequestedLine {
    data: Vec<f64>,
    width: u32,
    color: RGBColor,
    label: Option<String>
}

#[derive(Clone, Debug)]
struct Lines(Signal<Vec<RequestedLine>>);

#[component]
pub fn Line(data: Vec<f64>, width: Option<u32>, color: Option<RGBColor>, label: Option<String>) -> Element {
    let mut lines = consume_context::<Lines>().0;

    lines.with_mut(|v| v.push(RequestedLine { data, width: width.unwrap_or(3_u32), color: color.unwrap_or(RGBColor(255, 0, 0)), label }));

    rsx! {}
}

fn clear_canvas(canvas_id: &str) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id(canvas_id)
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    Ok(())
}

pub fn short_decimals(f: &f64) -> String {
    format!("{}", (f * 100.0).round() / 100.0)
}

#[component]
pub fn Chart(children: Element, formatter: fn(&f64) -> String) -> Element {
    let uuid = uuid::Uuid::new_v4();

    let lines: Signal<Vec<RequestedLine>> = use_signal(Vec::new);
    provide_context(Lines(lines));

    let all_values = lines.read().iter().fold(vec![], |acc, v| [acc, v.data.clone()].concat());
    let min = all_values.clone().into_iter().reduce(f64::min).unwrap_or(0.);
    let max = all_values.clone().into_iter().reduce(f64::max).unwrap_or(0.);

    let longest = use_memo(move || { lines.read().iter().map(|l| l.data.len()).max().unwrap_or(0) });

    let mut offset = use_signal(|| 0);

    let dark_mode: bool = web_sys::window().unwrap().match_media("(prefers-color-scheme: dark)").unwrap().unwrap().matches();

    let plot = move |lines_inner: Vec<RequestedLine>, offset: usize| {
        if let Some(backend) = CanvasBackend::new(&uuid.to_string()) {
            let root = backend.into_drawing_area();

            let _ = clear_canvas(&uuid.to_string());

            let mut chart = ChartBuilder::on(&root)
                .margin_left(80u32)
                .margin_right(10u32)
                .margin_bottom(10u32)
                .margin_top(10u32)
                .x_label_area_size(20u32)
                .y_label_area_size(20u32)
                .build_cartesian_2d(offset..32 + offset, min..max).unwrap();

            chart.configure_mesh()
                .disable_mesh()
                .axis_style(if dark_mode { &GREY_200 } else { &GREY_800 })
                .y_label_style(("sans-serif", 20, if dark_mode { &GREY_200 } else { &GREY_800 }))
                .x_label_style(("sans-serif", 20, if dark_mode { &GREY_200 } else { &GREY_800 }))
                .y_label_formatter(&formatter)
                .draw().unwrap();

            for line in lines_inner {
                let draw_call = chart.draw_series(LineSeries::new(
                    (offset..if 31 + offset >= line.data.len() { line.data.len() } else { offset + 31 })
                        .map(|i| (i, line.data[i])),
                    ShapeStyle::from(line.color).stroke_width(line.width),
                )).unwrap();
                if let Some(string) = line.label {
                    draw_call.label(string)
                        .legend(move |(x, y)| PathElement::new(vec![(x, y - 7), (x + 20, y - 7)], ShapeStyle::from(line.color)));
                }
            }

            chart.configure_series_labels()
                .border_style(if dark_mode { &WHITE } else { &BLACK })
                .background_style(if dark_mode { WHITE.mix(0.8) } else { BLACK.mix(0.1) })
                .label_font(("sans-serif", 20.0))
                .draw()
                .unwrap();
        } else {
            tracing::error!("Failed to instantiate CanvasBackend.");
        }
    };

    use_effect(move || {
        plot(lines(), offset())
    });

    rsx! {
        link { rel: "stylesheet", href: asset!("./style.css") }
        canvas { id: uuid.to_string(), width: 700, height: 350 }
        if longest() > 32 {
            span { class: "max-w-full flex flex-col",
                input {
                    class: "range",
                    r#type: "range",
                    margin_left: "14%",
                    margin_right: "1.4%",
                    value: offset(),
                    onchange: move |evt: Event<FormData>| {
                        let v = evt.value().parse::<usize>().unwrap_or(0);
                        offset.set(v);
                    },
                    oninput: move |evt: Event<FormData>| {
                        let v = evt.value().parse::<usize>().unwrap_or(0);
                        offset.set(v);
                    },
                    min: 0,
                    max: longest().saturating_sub(32),
                }
            }
        }
        {children}
    }
}
