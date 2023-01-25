#![recursion_limit = "512"]

use yew::prelude::*;

mod game;
mod util;

use crate::game::{Hand, Round, State};

struct Model {
    state: State,
}

enum Msg {
    HumanThrow(Hand),
    Restart,
    InspectRound(usize),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: State::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HumanThrow(hand) => {
                self.state.human_throw(hand);
            }
            Msg::Restart => {
                self.state = State::new();
            }
            Msg::InspectRound(i) => {
                self.state.set_selected_round(i);
            }
        }
        true
    }

    fn changed(&mut self, _ctx: &Context<Self>, _props: &Self::Properties) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="main jumbotron">
                <div class="row">
                    <div class="col-md-6">
                        <div class="row">
                            <div class="col-md">
                                { self.view_hands(ctx) }
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-md">
                                {
                                    if let Some(current_round) = self.state.current_round() {
                                        self.view_current_round(current_round)
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-md">
                                { self.view_scoreboard(ctx) }
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6">
                        <div class="row">
                            <div class="col-md">
                                {
                                    if self.state.selected_round().is_some() {
                                        self.view_round(self.state.selected_round(), false)
                                    } else {
                                        self.view_round(self.state.last_round(), true)
                                    }
                                }
                            </div>
                        </div>
                        <div class="row">
                            <div class="col-md">
                                { self.view_history(ctx) }
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.state.selected_round().is_none() {
            // Scrolls to the last row of the table.
            let document = web_sys::window()
                .expect("window")
                .document()
                .expect("document");
            let table_body = document
                .query_selector("#history tbody")
                .expect("query_selector")
                .expect("tbody");
            table_body.set_scroll_top(table_body.scroll_height());
        }
    }
}

impl Model {
    fn view_hands(&self, ctx: &Context<Self>) -> Html {
        let computer = match self.state.last_human_vs_computer() {
            Some((_human, computer)) => computer.as_icon(),
            None => "👌🏼",
        };

        html! {
            <div class="card border-primary mb-3">
                <div class="card-header text-primary border-primary">
                    <h3>{ "WASM-rock-paper-scissors" }</h3>
                </div>
                <div class="card-body text-primary button-container">
                    <button type="button" class="btn btn-secondary btn-hand">
                        { computer }
                    </button>
                    <label class="vs-label">{ "V.S." }</label>
                    <span class="btn-group btn-group-toggle" data-toggle="buttons">
                        <label class="btn btn-primary btn-hand">
                            <input type="radio" name="options" autocomplete="off"
                                onclick={ctx.link().callback(|_| Msg::HumanThrow(Hand::Rock))}/>
                            { Hand::Rock.as_icon() }
                        </label>

                        <label class="btn btn-primary btn-hand">
                            <input type="radio" name="options" autocomplete="off"
                                onclick={ctx.link().callback(|_| Msg::HumanThrow(Hand::Paper))}/>
                            { Hand::Paper.as_icon() }
                        </label>

                        <label class="btn btn-primary btn-hand">
                            <input type="radio" name="options" autocomplete="off"
                                onclick={ctx.link().callback(|_| Msg::HumanThrow(Hand::Scissors))}/>
                            { Hand::Scissors.as_icon() }
                        </label>
                    </span>
                </div>
            </div>
        }
    }

    fn view_scoreboard(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="scoreboard" class="card border-info mb-3">
                <div class="card-header text-info bg-transparent border-info">
                    { "Scoreboard" }
                </div>
                <div class="card-body text-info">
                    { "Win = " } { self.state.win_count() }
                    { ", Draw = " } { self.state.draw_count() }
                    { ", Loss = " } { self.state.loss_count() }
                    { "\u{00a0}\u{00a0}\u{00a0}" }
                    <button type="button" class="btn btn-info"
                        onclick={ctx.link().callback(|_| Msg::Restart)}>
                        { "Restart" }
                    </button>
                </div>
            </div>
        }
    }

    fn view_current_round(&self, current_round: &Round) -> Html {
        html! {
            <div id="current-round" class="card border-success mb-3">
                <div class="card-header text-success bg-transparent border-success">
                    { format!("Current round (#{})", current_round.num1()) }
                </div>
                <p class="card-body text-success">
                    { "The computer has picked a shape by claiming " }
                    <code>{ &current_round.digest }</code>
                    { "." }
                </p>
            </div>
        }
    }

    // Displays the last round, or the currently inspected round.
    fn view_round(&self, r: Option<&Round>, is_last: bool) -> Html {
        let i1 = r.map(|r| r.num1()).unwrap_or(0);
        html! {
            <div id="last-round" class="card border-warning mb-3">
                <div class="card-header text-warning bg-transparent border-warning">
                    {
                        if is_last {
                            format!("Last round (#{})", i1)
                        } else {
                            format!("Round #{}", i1)
                        }
                    }
                </div>
                <div class="card-body text-warning">{
                    match r {
                        Some(r) => self.view_round_impl(r),
                        None => html! { <div>{ "N/A" }</div> },
                    }
                }</div>
            </div>
        }
    }

    fn view_round_impl(&self, r: &Round) -> Html {
        let human = r
            .human
            .expect("Human did not throw at last round")
            .as_icon();
        let computer = r.computer.as_icon();
        let computer_str = r.computer.as_ref();
        let random = &r.random_bytes;
        let digest = &r.digest;
        let result = r.result_str().unwrap();

        html! {
            <div>
                <p>{
                    format!("{} V.S. {} ➯ {}",
                        human,
                        computer,
                        result)
                }</p>
                <p>
                    { "You can verify this by running "}
                    <code>{
                        format!("echo -n {}_{} | shasum -a 256",
                            random,
                            computer_str)
                    }</code>
                    <br/>
                    { "and check whether the output is "}
                    <code>{ digest }</code>
                    { "." }
                </p>
            </div>
        }
    }

    fn view_history(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="card border-secondary mb-3">
                <div class="card-header text-secondary bg-transparent border-secondary">
                    { "History" }
                </div>
                <div class="table-container card-body text-black">
                    <table class="table table-striped table-sm" id="history">
                        <thead>
                            <tr>
                                <td>{ "#" }</td>
                                <td>{ "You" }</td>
                                <td>{ "Computer" }</td>
                                <td>{ "Result" }</td>
                                <td>{ "Random" }</td>
                                <td>{ "Digest" }</td>
                            </tr>
                        </thead>
                        <tbody>
                            { for self
                                .state
                                .history()
                                .iter()
                                .map(|r| self.view_history_row(ctx, r)) }
                        </tbody>
                    </table>
                </div>
            </div>
        }
    }

    fn view_history_row(&self, ctx: &Context<Self>, r: &Round) -> Html {
        let id = r.i;
        html! {
            <tr onclick={ctx.link().callback(move |_| Msg::InspectRound(id))}>
                <td>{ r.num1() }</td>
                <td>{ r.human.unwrap().as_icon() }</td>
                <td>{ r.computer.as_icon() }</td>
                <td>{ Hand::cmp_as_char(r.human.unwrap(), r.computer) }</td>
                <td><pre>{ &r.random_bytes[..8] }</pre></td>
                <td><pre>{ &r.digest[..8] }</pre></td>
            </tr>
        }
    }
}

fn main() {
    yew::Renderer::<Model>::new().render();
}
