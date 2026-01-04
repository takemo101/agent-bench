use pomodoro::sound::{SoundSource, RodioSoundPlayer, SoundPlayer};

#[test]
fn test_discover_system_sounds_linux_container() {
    // In the container (Linux), /System/Library/Sounds usually doesn't exist.
    // So it should return empty list.
    let sounds = SoundSource::discover_system_sounds();
    
    // Check if we are really on linux and path doesn't exist
    if !std::path::Path::new("/System/Library/Sounds").exists() {
        assert!(sounds.is_empty(), "Should be empty when directory is missing");
    } else {
        // If it exists (e.g. maybe mounted?), just check it returns something or nothing but doesn't crash
        // For now assume container environment behavior.
    }
}

#[test]
fn test_get_default_source() {
    let source = SoundSource::get_default_source();
    match source {
        SoundSource::Embedded { name } => {
            assert_eq!(name, "default");
        },
        _ => panic!("Default should be Embedded when system sounds missing"),
    }
}

#[tokio::test]
async fn test_rodio_player_disabled() {
    // new(true) should succeed even without audio device
    let player = RodioSoundPlayer::new(true).expect("Should create disabled player");
    assert!(!player.is_available());
    
    // play should return Ok (silent success)
    let source = SoundSource::Embedded { name: "default".to_string() };
    let result = player.play(&source).await;
    assert!(result.is_ok());
}
