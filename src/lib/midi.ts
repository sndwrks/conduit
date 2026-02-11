const NOTE_NAMES = [
  "C",
  "C#",
  "D",
  "D#",
  "E",
  "F",
  "F#",
  "G",
  "G#",
  "A",
  "A#",
  "B",
];

/** Convert MIDI note number (0-127) to note name. Middle C = C3 = 60. */
export function midiNoteToName(note: number): string {
  if (note < 0 || note > 127 || !Number.isInteger(note)) return `${note}`;
  const octave = Math.floor(note / 12) - 2;
  const name = NOTE_NAMES[note % 12];
  return `${name}${octave}`;
}

/** Convert note name (e.g. "C3", "F#4") to MIDI note number. Returns null if invalid. */
export function noteNameToMidi(name: string): number | null {
  const match = name.match(/^([A-Ga-g])(#|b)?(-?\d+)$/);
  if (!match) return null;

  const letter = match[1].toUpperCase();
  const accidental = match[2] || "";
  const octave = parseInt(match[3], 10);

  const baseNotes: Record<string, number> = {
    C: 0,
    D: 2,
    E: 4,
    F: 5,
    G: 7,
    A: 9,
    B: 11,
  };

  const base = baseNotes[letter];
  if (base === undefined) return null;

  let semitone = base;
  if (accidental === "#") semitone += 1;
  if (accidental === "b") semitone -= 1;

  const midi = (octave + 2) * 12 + semitone;
  if (midi < 0 || midi > 127) return null;
  return midi;
}

/** Validate an OSC address — must start with / and contain only valid OSC characters */
export function isValidOscAddress(addr: string): boolean {
  return /^\/[a-zA-Z0-9_/.*?\[\]{},# -]*$/.test(addr);
}
