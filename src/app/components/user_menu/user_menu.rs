use gio::{ActionMapExt, SimpleAction, SimpleActionGroup};
use gtk::{AboutDialogExt, MenuButtonExt, WidgetExt};
use std::rc::Rc;

use super::UserMenuModel;
use crate::app::components::EventListener;
use crate::app::AppEvent;

pub struct UserMenu {
    user_button: gtk::MenuButton,
    about: gtk::AboutDialog,
    model: Rc<UserMenuModel>,
    configured: bool,
}

impl UserMenu {
    pub fn new(
        user_button: gtk::MenuButton,
        about: gtk::AboutDialog,
        model: UserMenuModel,
    ) -> Self {
        let model = Rc::new(model);

        about.set_version(Some(crate::VERSION));
        about.connect_delete_event(
            clone!(@weak about => @default-return gtk::Inhibit(false), move |_, _| {
                about.hide();
                gtk::Inhibit(true)
            }),
        );

        Self {
            user_button,
            about,
            model,
            configured: false,
        }
    }

    fn configure_actions_if_needed(&mut self) {
        if !self.configured {
            let action_group = SimpleActionGroup::new();

            action_group.add_action(&{
                let logout = SimpleAction::new("logout", None);
                logout.connect_activate(clone!(@weak self.model as model => move |_, _| {
                    model.logout();
                }));
                logout
            });

            action_group.add_action(&{
                let about = SimpleAction::new("about", None);
                about.connect_activate(clone!(@weak self.about as about => move |_, _| {
                    about.show_all();
                }));
                about
            });

            self.user_button
                .insert_action_group("menu", Some(&action_group));
            self.configured = true;
        }
    }

    fn update_menu(&self) {
        let menu = gio::Menu::new();
        menu.insert(0, Some("About"), Some("menu.about"));

        if let Some(username) = self.model.username() {
            let user_menu = gio::Menu::new();
            user_menu.insert(0, Some("Log out"), Some("menu.logout"));
            menu.insert_section(0, Some(&username), &user_menu);
        }

        self.user_button.set_menu_model(Some(&menu));
    }
}

impl EventListener for UserMenu {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::LoginCompleted(_) => {
                self.configure_actions_if_needed();
                self.update_menu();
            }
            _ => {}
        }
    }
}