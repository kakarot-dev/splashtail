use futures_util::stream::{Stream, StreamExt};
use mlua::prelude::*;
use std::pin::Pin;

pub struct LuaStream<T: Stream<Item: IntoLua + Send> + Send + 'static> {
    pub inner: Pin<Box<T>>, // Box the stream to ensure its pinned,
}

impl<ST: Stream<Item: IntoLua + Send> + Send + 'static> LuaStream<ST> {
    pub fn new(stream: ST) -> Self {
        Self {
            inner: Box::pin(stream), // Pin the stream
        }
    }
}

impl<T: Stream<Item: IntoLua + Send> + Send + 'static> LuaUserData for LuaStream<T> {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Go to the next item in the stream
        methods.add_async_method_mut("next", |lua, mut this, _: ()| async move {
            match this.inner.next().await {
                Some(item) => Ok(item.into_lua(&lua)?), // Convert the item to LuaValue
                None => Ok(LuaValue::Nil),              // Return nil if the stream is exhausted
            }
        }); // Implement the method

        // Executes a callback for every entry in the stream
        methods.add_async_method_mut(
            "for_each",
            |lua, mut this, callback: LuaFunction| async move {
                while let Some(item) = this.inner.next().await {
                    let item_value = item.into_lua(&lua)?; // Convert the item to LuaValue
                    callback
                        .call_async::<()>((
                            item_value, // Convert the item to LuaValue
                        ))
                        .await?; // Call the Lua callback
                }
                Ok(())
            },
        );
    }
}
