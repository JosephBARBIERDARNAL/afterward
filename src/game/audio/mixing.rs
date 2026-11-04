use super::*;

pub fn update_audio_volume(
    time: Res<Time>,
    screen: Res<State<GameScreen>>,
    session: Res<RunSession>,
    data: Res<GameData>,
    settings: Res<AudioSettings>,
    mut audio_mix: Local<Option<(f32, f32, f32, f32)>>,
    mut active_audio: Query<(&ManagedAudio, Option<&mut AudioSink>)>,
) {
    let target_music_volume = match screen.get() {
        GameScreen::Paused | GameScreen::Settings => {
            data.gameplay.audio.paused_music_volume_multiplier
        }
        GameScreen::RoomClear | GameScreen::Bonus | GameScreen::BossIntro => {
            data.gameplay.audio.room_clear_music_volume_multiplier
        }
        _ => 1.0,
    };
    let calm_music_active =
        matches!(screen.get(), GameScreen::RoomClear | GameScreen::Bonus) || session.door_open;
    let target_combat_music_mix = if calm_music_active {
        0.0
    } else {
        target_music_volume
    };
    let target_calm_music_mix = if calm_music_active {
        target_music_volume
    } else {
        0.0
    };
    let target_ambience_mix = target_music_volume;
    let target_gameplay_effects_mix = if gameplay_effects_muted(*screen.get()) {
        0.0
    } else {
        1.0
    };
    let previous_mix = *audio_mix;
    let (
        current_combat_music_mix,
        current_calm_music_mix,
        current_ambience_mix,
        current_gameplay_effects_mix,
    ) = previous_mix.unwrap_or((
        target_combat_music_mix,
        target_calm_music_mix,
        target_ambience_mix,
        target_gameplay_effects_mix,
    ));
    let mix_speed = time.delta_secs() * 4.0;
    let next_combat_music_mix =
        move_towards(current_combat_music_mix, target_combat_music_mix, mix_speed);
    let next_calm_music_mix =
        move_towards(current_calm_music_mix, target_calm_music_mix, mix_speed);
    let next_ambience_mix = move_towards(current_ambience_mix, target_ambience_mix, mix_speed);
    let next_gameplay_effects_mix = move_towards(
        current_gameplay_effects_mix,
        target_gameplay_effects_mix,
        mix_speed,
    );
    *audio_mix = Some((
        next_combat_music_mix,
        next_calm_music_mix,
        next_ambience_mix,
        next_gameplay_effects_mix,
    ));
    let mix_changed = previous_mix.is_none_or(|(combat, calm, ambience, effects)| {
        (combat - next_combat_music_mix).abs() > 0.0001
            || (calm - next_calm_music_mix).abs() > 0.0001
            || (ambience - next_ambience_mix).abs() > 0.0001
            || (effects - next_gameplay_effects_mix).abs() > 0.0001
    });
    if !settings.is_changed() && !mix_changed {
        return;
    }
    for (audio, sink) in &mut active_audio {
        let Some(mut sink) = sink else {
            continue;
        };
        let (channel_volume, mix) = match audio.class {
            AudioClass::Music => (settings.music_volume_percent, next_combat_music_mix),
            AudioClass::CalmMusic => (settings.music_volume_percent, next_calm_music_mix),
            AudioClass::Ambience => (settings.music_volume_percent, next_ambience_mix),
            AudioClass::GameplayEffect => (settings.sfx_volume_percent, next_gameplay_effects_mix),
            AudioClass::InterfaceEffect | AudioClass::TerminalEffect => {
                (settings.sfx_volume_percent, 1.0)
            }
        };
        sink.set_volume(Volume::Linear(
            channel_volume as f32 / 100.0 * audio.gain * mix,
        ));
    }
}

fn gameplay_effects_muted(screen: GameScreen) -> bool {
    matches!(
        screen,
        GameScreen::Paused
            | GameScreen::Settings
            | GameScreen::MainMenu
            | GameScreen::Dead
            | GameScreen::Complete
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gameplay_audio_fades_outside_active_gameplay() {
        for screen in [
            GameScreen::MainMenu,
            GameScreen::Paused,
            GameScreen::Settings,
            GameScreen::Dead,
            GameScreen::Complete,
        ] {
            assert!(gameplay_effects_muted(screen));
        }

        for screen in [
            GameScreen::RoomIntro,
            GameScreen::Playing,
            GameScreen::RoomClear,
            GameScreen::Bonus,
            GameScreen::BossIntro,
            GameScreen::Transitioning,
            GameScreen::RoomLoading,
            GameScreen::TransitioningOut,
        ] {
            assert!(!gameplay_effects_muted(screen));
        }
    }
}
