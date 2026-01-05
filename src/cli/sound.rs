use crate::cli::commands::ConfigArgs;
use crate::sound::{config::SoundConfig, SoundSource};
use anyhow::Result;
use colored::Colorize;

pub fn handle_sounds() -> Result<()> {
    let sounds = SoundSource::discover_system_sounds();

    if sounds.is_empty() {
        println!("システムサウンドが見つかりませんでした。");
        return Ok(());
    }

    println!("{}", "利用可能なシステムサウンド:".bold());
    for sound in sounds {
        if let SoundSource::System { name, .. } = sound {
            println!("  - {}", name);
        } else if let SoundSource::Embedded { name } = sound {
            println!("  - {} (Embedded)", name);
        }
    }

    Ok(())
}

pub fn handle_config(args: ConfigArgs) -> Result<()> {
    let mut config =
        SoundConfig::load().map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    let mut updated = false;

    if let Some(sound) = args.work_sound {
        config.work_end_sound = sound;
        updated = true;
    }

    if let Some(sound) = args.break_sound {
        config.break_end_sound = sound;
        updated = true;
    }

    if updated {
        config
            .save()
            .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
        println!("{}", "設定を更新しました。".green());
    }

    // 現在の設定を表示
    println!("{}", "現在のサウンド設定:".bold());
    println!("  作業完了音: {}", config.work_end_sound);
    println!("  休憩完了音: {}", config.break_end_sound);

    Ok(())
}
