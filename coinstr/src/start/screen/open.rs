// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use coinstr_core::bitcoin::Network;
use coinstr_core::util::dir;
use coinstr_core::Coinstr;
use iced::widget::{column, row, svg, Column, PickList, Rule, Space};
use iced::{Alignment, Command, Element, Length};

use super::view;
use crate::component::{button, Text, TextInput};
use crate::constants::{APP_DESCRIPTION, APP_LOGO, APP_NAME};
use crate::start::{Context, Message, Stage, State};
use crate::theme::color::{DARK_RED, GREY};
use crate::KEYCHAINS_PATH;

#[derive(Debug, Clone)]
pub enum OpenMessage {
    LoadKeychains,
    NetworkSelect(Network),
    KeychainSelect(String),
    PasswordChanged(String),
    OpenButtonPressed,
}

#[derive(Debug)]
pub struct OpenState {
    network: Network,
    keychains: Vec<String>,
    name: Option<String>,
    password: String,
    error: Option<String>,
}

impl Default for OpenState {
    fn default() -> Self {
        Self {
            // TODO: add option to select network
            network: Network::Testnet,
            keychains: Vec::new(),
            name: None,
            password: String::new(),
            error: None,
        }
    }
}

impl OpenState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl State for OpenState {
    fn title(&self) -> String {
        format!("{APP_NAME} - Open")
    }

    fn load(&mut self, _ctx: &Context) -> Command<Message> {
        Command::perform(async {}, |_| Message::Open(OpenMessage::LoadKeychains))
    }

    fn update(&mut self, _ctx: &mut Context, message: Message) -> Command<Message> {
        if let Message::Open(msg) = message {
            match msg {
                OpenMessage::LoadKeychains => {
                    match dir::get_keychains_list(KEYCHAINS_PATH.as_path()) {
                        Ok(list) => self.keychains = list,
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                OpenMessage::NetworkSelect(network) => self.network = network,
                OpenMessage::KeychainSelect(name) => self.name = Some(name),
                OpenMessage::PasswordChanged(psw) => self.password = psw,
                OpenMessage::OpenButtonPressed => {
                    if let Some(name) = &self.name {
                        match dir::get_keychain_file(KEYCHAINS_PATH.as_path(), name) {
                            Ok(path) => match Coinstr::open(
                                path,
                                || Ok(self.password.clone()),
                                self.network,
                            ) {
                                Ok(keechain) => {
                                    return Command::perform(async {}, move |_| {
                                        Message::OpenResult(keechain)
                                    })
                                }
                                Err(e) => self.error = Some(e.to_string()),
                            },
                            Err(e) => self.error = Some(e.to_string()),
                        }
                    } else {
                        self.error = Some(String::from("Please, select a keychain"));
                    }
                }
            }
        };

        Command::none()
    }

    fn view(&self, _ctx: &Context) -> Element<Message> {
        let handle = svg::Handle::from_memory(APP_LOGO);
        let svg = svg(handle)
            .width(Length::Fixed(100.0))
            .height(Length::Fixed(100.0));

        let keychain_pick_list = Column::new()
            .push(Text::new("Keychain").view())
            .push(
                PickList::new(self.keychains.clone(), self.name.clone(), |name| {
                    Message::Open(OpenMessage::KeychainSelect(name))
                })
                .width(Length::Fill)
                .text_size(20)
                .padding(10)
                .placeholder(if self.keychains.is_empty() {
                    "No keychain availabe"
                } else {
                    "Select a keychain"
                }),
            )
            .spacing(5);

        let password = TextInput::new("Password", &self.password, |s| {
            Message::Open(OpenMessage::PasswordChanged(s))
        })
        .placeholder("Enter password")
        .on_submit(Message::Open(OpenMessage::OpenButtonPressed))
        .password()
        .view();

        let open_btn = button::primary("Open")
            .width(Length::Fill)
            .on_press(Message::Open(OpenMessage::OpenButtonPressed));

        let new_keychain_btn = button::border("Create keychain")
            .on_press(Message::View(Stage::New))
            .width(Length::Fill);
        let restore_keychain_btn = button::border("Restore keychain")
            .on_press(Message::View(Stage::Restore))
            .width(Length::Fill);

        let content = column![
            row![column![
                row![svg],
                row![Space::with_height(Length::Fixed(5.0))],
                row![Text::new(APP_DESCRIPTION).size(22).color(GREY).view()]
            ]
            .align_items(Alignment::Center)
            .spacing(15)],
            row![Space::with_height(Length::Fixed(10.0))],
            row![keychain_pick_list],
            row![password].spacing(10),
            if let Some(error) = &self.error {
                row![Text::new(error).color(DARK_RED).view()]
            } else {
                row![]
            },
            row![open_btn],
            row![Rule::horizontal(1)],
            row![new_keychain_btn],
            row![restore_keychain_btn],
        ];

        view(content)
    }
}

impl From<OpenState> for Box<dyn State> {
    fn from(s: OpenState) -> Box<dyn State> {
        Box::new(s)
    }
}
