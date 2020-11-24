use std::rc::Rc;
use std::cell::{RefCell, Ref};
use ref_filter_map::*;

use crate::app::{AppModel, AppAction, BrowserAction, ActionDispatcher};
use crate::app::dispatch::Worker;
use crate::app::models::*;
use crate::app::backend::api::SpotifyApiClient;
use crate::app::state::{ScreenName, LibraryState};

use super::*;

pub struct BrowserFactory {
    worker: Worker,
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>
}

impl BrowserFactory {

    pub fn new(
        worker: Worker,
        app_model: Rc<RefCell<AppModel>>,
        dispatcher: Box<dyn ActionDispatcher>) -> Self {

        Self { worker, app_model, dispatcher }
    }

    pub fn make_browser(&self) -> Browser {
        let model = BrowserModelImpl::new(Rc::clone(&self.app_model), self.dispatcher.box_clone());
        Browser::new(self.worker.clone(), Rc::new(model))
    }
}

pub struct BrowserModelImpl {
    app_model: Rc<RefCell<AppModel>>,
    dispatcher: Box<dyn ActionDispatcher>,
    batch_size: u32
}

impl BrowserModelImpl {

    pub fn new(app_model: Rc<RefCell<AppModel>>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        BrowserModelImpl { app_model, dispatcher, batch_size: 20 }
    }

    fn spotify(&self) -> Rc<dyn SpotifyApiClient> {
        Rc::clone(&self.app_model.borrow().services.spotify_api)
    }

    fn state(&self) -> Option<Ref<'_, LibraryState>> {
        ref_filter_map(self.app_model.borrow(), |m| m.state.browser_state.library_state())
    }
}

impl BrowserModel for BrowserModelImpl {

    fn get_saved_albums(&self) -> Option<Ref<'_, Vec<AlbumDescription>>> {
        Some(Ref::map(self.state()?, |s| &s.albums))
    }

    fn refresh_saved_albums(&self) {
        let api = self.spotify();
        let batch_size = self.batch_size;

        self.dispatcher.dispatch_async(Box::pin(async move {
            let albums = api.get_saved_albums(0, batch_size).await.unwrap_or(vec![]);
            Some(BrowserAction::SetContent(albums).into())
        }));
    }


    fn load_more_albums(&self) {
        let api = self.spotify();
        let page = self.state().map(|s| s.page).unwrap_or(0);
        let offset = page * self.batch_size;
        let batch_size = self.batch_size;

        self.dispatcher.dispatch_async(Box::pin(async move {
            let albums = api.get_saved_albums(offset, batch_size).await.unwrap_or(vec![]);
            Some(BrowserAction::AppendContent(albums).into())
        }));
    }

    fn play_album(&self, album_uri: &str) {
        let api = self.spotify();
        let uri = String::from(album_uri);

        self.dispatcher.dispatch_many_async(Box::pin(async move {
            if let Some(album) = api.get_album(&uri).await {
                let first_song = album.songs[0].uri.clone();
                vec![
                    AppAction::LoadPlaylist(album.songs),
                    AppAction::Load(first_song.clone())
                ]
            } else {
                vec![]
            }
        }));
    }

    fn open_album(&self, album_uri: &str) {
        let screen = ScreenName::Details(album_uri.to_owned());
        self.dispatcher.dispatch(BrowserAction::NavigationPush(screen).into());

        let album = self.get_saved_albums().and_then(|albums| {
            albums.iter().find(|a| a.id.eq(album_uri)).cloned()
        });

        if let Some(album) = album {
            self.dispatcher.dispatch(BrowserAction::SetDetails(album).into());
        }
    }
}