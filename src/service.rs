use std::os::unix::net::UnixStream;

use crate::domain::{Command, Property, PropertyValue, Result};
use anyhow::anyhow;

pub fn toggle_pause(stream: &mut UnixStream) -> Result<()> {
    let pause_status = is_paused(stream)?;
    Command::SetProperty(Property::Pause, PropertyValue::Bool(!pause_status))
        .execute(stream)
        .map(|_| ())
}

pub fn is_paused(stream: &mut UnixStream) -> Result<bool> {
    Command::GetProperty(Property::Pause)
        .execute(stream)?
        .data
        .ok_or(anyhow!("Failed to get pause status from response"))
        .and_then(TryInto::try_into)
}

pub fn play_next_chapter(stream: &mut UnixStream) -> Result<()> {
    let current_chapter =
        get_current_chapter(stream).ok_or(anyhow!("Current chapter not found"))?;
    Command::SetProperty(
        Property::Chapter,
        PropertyValue::Integer(current_chapter + 1),
    )
    .execute(stream)
    .map(|_| ())
}

pub fn play_previous_chapter(stream: &mut UnixStream) -> Result<()> {
    let current_chapter =
        get_current_chapter(stream).ok_or(anyhow!("Current chapter not found"))?;
    Command::SetProperty(
        Property::Chapter,
        PropertyValue::Integer(i32::min(current_chapter - 1, 0)),
    )
    .execute(stream)
    .map(|_| ())
}

pub fn get_current_chapter(stream: &mut UnixStream) -> Option<i32> {
    Command::GetProperty(Property::Chapter)
        .execute(stream)
        .ok()?
        .data
        .and_then(|chapter| chapter.try_into().ok())
}

pub fn get_total_chapters(stream: &mut UnixStream) -> Option<i32> {
    Command::GetProperty(Property::Chapters)
        .execute(stream)
        .ok()?
        .data
        .and_then(|chapter| chapter.try_into().ok())
        .and_then(|chapter: i32| if chapter == 0 { None } else { Some(chapter) })
}

pub fn get_clean_media_title(stream: &mut UnixStream) -> Result<String> {
    get_raw_media_title(stream)?
        .split(['-', '|', '/'])
        .take(1)
        .next()
        .map(|s| s.trim().replace('&', "and"))
        .ok_or(anyhow!("Got empty media title"))
}

pub fn get_raw_media_title(stream: &mut UnixStream) -> Result<String> {
    Command::GetProperty(Property::MediaTitle)
        .execute(stream)?
        .data
        .ok_or(anyhow!("Media title not found in response"))
        .and_then(TryInto::try_into)
}

pub fn increase_volume(stream: &mut UnixStream, mut value: i32) -> Result<()> {
    let current_volume = get_volume(stream)?;
    let needed_for_max = 100 - current_volume;

    if needed_for_max < value {
        value = needed_for_max;
    }

    if value == 0 {
        return Ok(());
    }

    Command::SetProperty(
        Property::Volume,
        PropertyValue::Integer(current_volume + value),
    )
    .execute(stream)
    .map(|_| ())
}

pub fn decrease_volume(stream: &mut UnixStream, mut value: i32) -> Result<()> {
    let current_volume = get_volume(stream)?;
    let needed_for_min = current_volume - value;

    if needed_for_min < 0 {
        value = current_volume;
    }

    if value == 0 {
        return Ok(());
    }

    Command::SetProperty(
        Property::Volume,
        PropertyValue::Integer(current_volume - value),
    )
    .execute(stream)
    .map(|_| ())
}

pub fn get_volume(stream: &mut UnixStream) -> Result<i32> {
    Command::GetProperty(Property::Volume)
        .execute(stream)?
        .data
        .ok_or(anyhow!("some"))?
        .try_into()
}
