import { describe, it, expect } from "vitest";
import {
  validatePort,
  validateMidiValue,
  validateMidiNote,
  validateOscFloat,
  validateOscInt,
} from "./validators";

describe("validatePort", () => {
  it("accepts valid ports", () => {
    expect(validatePort("1024")).toEqual({ valid: true, value: 1024 });
    expect(validatePort("8080")).toEqual({ valid: true, value: 8080 });
    expect(validatePort("65535")).toEqual({ valid: true, value: 65535 });
  });

  it("rejects ports below 1024", () => {
    expect(validatePort("1023")).toEqual({ valid: false });
    expect(validatePort("0")).toEqual({ valid: false });
    expect(validatePort("80")).toEqual({ valid: false });
  });

  it("rejects ports above 65535", () => {
    expect(validatePort("65536")).toEqual({ valid: false });
    expect(validatePort("100000")).toEqual({ valid: false });
  });

  it("rejects non-integers", () => {
    expect(validatePort("8080.5")).toEqual({ valid: false });
    expect(validatePort("abc")).toEqual({ valid: false });
    expect(validatePort("")).toEqual({ valid: false });
    expect(validatePort(" ")).toEqual({ valid: false });
  });

  it("trims whitespace", () => {
    expect(validatePort(" 8080 ")).toEqual({ valid: true, value: 8080 });
  });
});

describe("validateMidiValue", () => {
  it("accepts 0-127", () => {
    expect(validateMidiValue("0")).toEqual({ valid: true, value: 0 });
    expect(validateMidiValue("64")).toEqual({ valid: true, value: 64 });
    expect(validateMidiValue("127")).toEqual({ valid: true, value: 127 });
  });

  it("rejects out of range", () => {
    expect(validateMidiValue("-1")).toEqual({ valid: false });
    expect(validateMidiValue("128")).toEqual({ valid: false });
  });

  it("rejects non-integers", () => {
    expect(validateMidiValue("64.5")).toEqual({ valid: false });
    expect(validateMidiValue("abc")).toEqual({ valid: false });
    expect(validateMidiValue("")).toEqual({ valid: false });
  });
});

describe("validateMidiNote", () => {
  it("accepts integers 0-127", () => {
    expect(validateMidiNote("0")).toEqual({ valid: true, value: 0 });
    expect(validateMidiNote("60")).toEqual({ valid: true, value: 60 });
    expect(validateMidiNote("127")).toEqual({ valid: true, value: 127 });
  });

  it("accepts note names", () => {
    expect(validateMidiNote("C3")).toEqual({ valid: true, value: 60 });
    expect(validateMidiNote("F#4")).toEqual({ valid: true, value: 78 });
    expect(validateMidiNote("Bb4")).toEqual({ valid: true, value: 82 });
    expect(validateMidiNote("c3")).toEqual({ valid: true, value: 60 });
  });

  it("rejects out of range", () => {
    expect(validateMidiNote("-1")).toEqual({ valid: false });
    expect(validateMidiNote("128")).toEqual({ valid: false });
  });

  it("rejects invalid strings", () => {
    expect(validateMidiNote("")).toEqual({ valid: false });
    expect(validateMidiNote("abc")).toEqual({ valid: false });
    expect(validateMidiNote("X3")).toEqual({ valid: false });
  });
});

describe("validateOscFloat", () => {
  it("accepts valid floats", () => {
    expect(validateOscFloat("0")).toEqual({ valid: true, value: 0 });
    expect(validateOscFloat("0.5")).toEqual({ valid: true, value: 0.5 });
    expect(validateOscFloat("-1.5")).toEqual({ valid: true, value: -1.5 });
    expect(validateOscFloat("100")).toEqual({ valid: true, value: 100 });
  });

  it("rejects non-numeric", () => {
    expect(validateOscFloat("abc")).toEqual({ valid: false });
    expect(validateOscFloat("")).toEqual({ valid: false });
  });

  it("rejects Infinity and NaN", () => {
    expect(validateOscFloat("Infinity")).toEqual({ valid: false });
    expect(validateOscFloat("NaN")).toEqual({ valid: false });
  });
});

describe("validateOscInt", () => {
  it("accepts valid integers", () => {
    expect(validateOscInt("0")).toEqual({ valid: true, value: 0 });
    expect(validateOscInt("-10")).toEqual({ valid: true, value: -10 });
    expect(validateOscInt("42")).toEqual({ valid: true, value: 42 });
  });

  it("rejects floats", () => {
    expect(validateOscInt("1.5")).toEqual({ valid: false });
    expect(validateOscInt("0.1")).toEqual({ valid: false });
  });

  it("rejects non-numeric", () => {
    expect(validateOscInt("abc")).toEqual({ valid: false });
    expect(validateOscInt("")).toEqual({ valid: false });
  });
});
