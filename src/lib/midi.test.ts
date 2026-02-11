import { describe, it, expect } from "vitest";
import { midiNoteToName, noteNameToMidi, isValidOscAddress } from "./midi";

describe("midiNoteToName", () => {
  it("converts middle C (60) to C3", () => {
    expect(midiNoteToName(60)).toBe("C3");
  });

  it("converts note 0 to C-2", () => {
    expect(midiNoteToName(0)).toBe("C-2");
  });

  it("converts note 127 to G8", () => {
    expect(midiNoteToName(127)).toBe("G8");
  });

  it("converts note 69 (A3) correctly", () => {
    expect(midiNoteToName(69)).toBe("A3");
  });

  it("handles sharps", () => {
    expect(midiNoteToName(61)).toBe("C#3");
    expect(midiNoteToName(66)).toBe("F#3");
  });

  it("returns number string for invalid input", () => {
    expect(midiNoteToName(-1)).toBe("-1");
    expect(midiNoteToName(128)).toBe("128");
  });
});

describe("noteNameToMidi", () => {
  it("converts C3 to 60", () => {
    expect(noteNameToMidi("C3")).toBe(60);
  });

  it("converts C-2 to 0", () => {
    expect(noteNameToMidi("C-2")).toBe(0);
  });

  it("converts G8 to 127", () => {
    expect(noteNameToMidi("G8")).toBe(127);
  });

  it("handles sharps", () => {
    expect(noteNameToMidi("C#3")).toBe(61);
    expect(noteNameToMidi("F#4")).toBe(66 + 12);
  });

  it("handles flats", () => {
    expect(noteNameToMidi("Db3")).toBe(61);
    expect(noteNameToMidi("Bb4")).toBe(82);
  });

  it("is case-insensitive for letter", () => {
    expect(noteNameToMidi("c3")).toBe(60);
  });

  it("returns null for invalid input", () => {
    expect(noteNameToMidi("")).toBeNull();
    expect(noteNameToMidi("X3")).toBeNull();
    expect(noteNameToMidi("C")).toBeNull();
    expect(noteNameToMidi("hello")).toBeNull();
  });

  it("round-trips with midiNoteToName", () => {
    for (let i = 0; i <= 127; i++) {
      const name = midiNoteToName(i);
      const back = noteNameToMidi(name);
      expect(back).toBe(i);
    }
  });
});

describe("isValidOscAddress", () => {
  it("accepts valid OSC addresses", () => {
    expect(isValidOscAddress("/cue/go")).toBe(true);
    expect(isValidOscAddress("/")).toBe(true);
    expect(isValidOscAddress("/foo/bar/baz")).toBe(true);
  });

  it("accepts OSC pattern characters", () => {
    expect(isValidOscAddress("/cue/*")).toBe(true);
    expect(isValidOscAddress("/cue/[1-3]")).toBe(true);
    expect(isValidOscAddress("/cue/{go,stop}")).toBe(true);
    expect(isValidOscAddress("/cue?")).toBe(true);
    expect(isValidOscAddress("/#bundle")).toBe(true);
  });

  it("rejects addresses not starting with /", () => {
    expect(isValidOscAddress("cue/go")).toBe(false);
    expect(isValidOscAddress("")).toBe(false);
  });

  it("rejects addresses with control characters", () => {
    expect(isValidOscAddress("/cue\x00go")).toBe(false);
    expect(isValidOscAddress("/cue\ngo")).toBe(false);
    expect(isValidOscAddress("/cue\tgo")).toBe(false);
  });
});
