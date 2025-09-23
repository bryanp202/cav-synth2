use std::sync::mpsc::Sender;
use crate::audio::AudioMessage;

use midir::MidiInput;
use midir::Ignore;
use midir::MidiInputConnection;

pub fn setup_midi(output: Sender<AudioMessage>) -> Result<MidiInputConnection<()>, String> {
    let mut midi_in = MidiInput::new("cav-synth").map_err(|err| err.to_string())?;
    midi_in.ignore(Ignore::TimeAndActiveSense);

    let in_ports = midi_in.ports();
    println!("Midi port count: {}", in_ports.len());
    if in_ports.len() < 1 {
        return Err(String::from("No midi ports found"));
    }
    let in_port = &in_ports[0];

    midi_in.connect(
        in_port, 
        "synth-midi", 
        move |_stamp, message, _| {
            match message[0] {
                144 => { // Key press / key release
                    if message[2] != 0 {
                        output.send(AudioMessage::KeyPress(message[1], message[2])).unwrap();
                    } else {
                        output.send(AudioMessage::KeyRelease(message[1])).unwrap();
                    }
                }

                176 => { // Pedal press
                    match message[1] {
                        64 => {
                            if message[2] == 0 {
                                output.send(AudioMessage::PedalRelease).unwrap();
                            } else {
                                output.send(AudioMessage::PedalPress).unwrap();
                            }
                        }
                        _ => ()
                    }
                }

                _ => (),
            }
        },
        (),
    ).map_err(|err| err.to_string())
}