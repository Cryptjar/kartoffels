use anyhow::anyhow;
use futures::StreamExt;
use glam::{ivec2, IVec2};
use kartoffels::prelude::*;
use serde::ser::Serialize;
use serde_wasm_bindgen::Serializer;
use std::borrow::Cow;
use std::panic;
use tracing::{subscriber, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use tracing_wasm::{WASMLayer, WASMLayerConfigBuilder};
use wasm_bindgen::prelude::*;
use wasm_streams::readable::sys;
use wasm_streams::ReadableStream;

type Result<T> = anyhow::Result<T, JsError>;

#[wasm_bindgen]
pub struct Sandbox {
    handle: Handle,
    spawn_point: Option<IVec2>,
}

#[wasm_bindgen]
impl Sandbox {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<Sandbox> {
        let config = serde_wasm_bindgen::from_value(config)
            .map_err(|err| anyhow!("couldn't parse config: {:?}", err))
            .into_js_error()?;

        let handle = kartoffels::create(WorldId::SANDBOX, config, None)
            .into_js_error()?;

        Ok(Self {
            handle,
            spawn_point: None,
        })
    }

    pub async fn listen(&self) -> Result<sys::ReadableStream> {
        let stream = self
            .handle
            .listen()
            .await
            .into_js_error()?
            .map(|val| Ok(val.into_js_value()));

        Ok(ReadableStream::from_stream(stream).into_raw())
    }

    pub async fn join(
        &self,
        id: Option<String>,
    ) -> Result<sys::ReadableStream> {
        let id = id.map(|id| id.parse()).transpose().into_js_error()?;

        let stream = self
            .handle
            .join(id)
            .await
            .into_js_error()?
            .map(|val| Ok(val.into_js_value()));

        Ok(ReadableStream::from_stream(stream).into_raw())
    }

    pub async fn pause(&self, paused: bool) -> Result<()> {
        self.handle.pause(paused).await.into_js_error()?;

        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        self.handle.close().await.into_js_error()?;

        Ok(())
    }

    pub async fn create_bot(&self, src: Vec<u8>) -> Result<JsValue> {
        let id = self
            .handle
            .create_bot(Cow::Owned(src), self.spawn_point, false)
            .await
            .into_js_error()?
            .into_js_value();

        Ok(id)
    }

    pub async fn create_prefab_bot(
        &self,
        ty: String,
        x: Option<i32>,
        y: Option<i32>,
        ephemeral: Option<bool>,
    ) -> Result<JsValue> {
        let src: &[u8] = match ty.as_str() {
            "dummy" => {
                include_bytes!(env!("KARTOFFELS_BOT_DUMMY"))
            }
            "roberto" => {
                include_bytes!(env!("KARTOFFELS_BOT_ROBERTO"))
            }
            _ => {
                return Err(JsError::new("unknown prefab"));
            }
        };

        let at = Self::decode_opt_pos(x, y).or(self.spawn_point);
        let ephemeral = ephemeral.unwrap_or(false);

        let id = self
            .handle
            .create_bot(Cow::Borrowed(src), at, ephemeral)
            .await
            .into_js_error()?
            .into_js_value();

        Ok(id)
    }

    pub async fn restart_bot(&self, id: String) -> Result<()> {
        let id = id.parse().into_js_error()?;

        self.handle.restart_bot(id).await.into_js_error()?;

        Ok(())
    }

    pub async fn destroy_bot(&self, id: String) -> Result<()> {
        let id = id.parse().into_js_error()?;

        self.handle.destroy_bot(id).await.into_js_error()?;

        Ok(())
    }

    pub async fn get_bots(&self) -> Result<JsValue> {
        Ok(self
            .handle
            .get_bots()
            .await
            .into_js_error()?
            .into_js_value())
    }

    pub fn set_spawn_point(
        &mut self,
        x: Option<i32>,
        y: Option<i32>,
    ) -> Result<()> {
        self.spawn_point = Self::decode_opt_pos(x, y);

        Ok(())
    }

    fn decode_opt_pos(x: Option<i32>, y: Option<i32>) -> Option<IVec2> {
        if x.is_none() && y.is_none() {
            None
        } else {
            Some(ivec2(x.unwrap_or_default(), y.unwrap_or_default()))
        }
    }
}

trait IntoJsError<T> {
    fn into_js_error(self) -> Result<T>;
}

impl<T> IntoJsError<T> for anyhow::Result<T> {
    fn into_js_error(self) -> Result<T> {
        self.map_err(|err| JsError::new(&format!("{:?}", err)))
    }
}

trait IntoJsValue {
    fn into_js_value(self) -> JsValue;
}

impl<T> IntoJsValue for T
where
    T: Serialize,
{
    fn into_js_value(self) -> JsValue {
        self.serialize(&Serializer::new().serialize_maps_as_objects(true))
            .unwrap()
    }
}

#[wasm_bindgen(start)]
fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let enable_logs = web_sys::window()
        .expect("couldn't find window")
        .get("enableSandboxLogs")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if enable_logs {
        subscriber::set_global_default(
            Registry::default().with(WASMLayer::new(
                WASMLayerConfigBuilder::new()
                    .set_max_level(Level::INFO)
                    .build(),
            )),
        )
        .unwrap();
    }
}
