import { noteNameToMidi } from "@/lib/midi";

export type ValidationResult<T = number> =
  | { valid: true; value: T }
  | { valid: false };

export function validatePort(raw: string): ValidationResult {
  const trimmed = raw.trim();
  if (trimmed === "") return { valid: false };
  const num = Number(trimmed);
  if (!Number.isInteger(num) || num < 1024 || num > 65535) return { valid: false };
  return { valid: true, value: num };
}

export function validateMidiValue(raw: string): ValidationResult {
  const trimmed = raw.trim();
  if (trimmed === "") return { valid: false };
  const num = Number(trimmed);
  if (!Number.isInteger(num) || num < 0 || num > 127) return { valid: false };
  return { valid: true, value: num };
}

export function validateMidiNote(raw: string): ValidationResult {
  const trimmed = raw.trim();
  if (trimmed === "") return { valid: false };
  // Try as integer first
  const num = Number(trimmed);
  if (Number.isInteger(num) && num >= 0 && num <= 127) {
    return { valid: true, value: num };
  }
  // Try as note name
  const midi = noteNameToMidi(trimmed);
  if (midi !== null) return { valid: true, value: midi };
  return { valid: false };
}

export function validateOscFloat(raw: string): ValidationResult {
  const trimmed = raw.trim();
  if (trimmed === "") return { valid: false };
  const num = Number(trimmed);
  if (!Number.isFinite(num)) return { valid: false };
  return { valid: true, value: num };
}

export function validateOscInt(raw: string): ValidationResult {
  const trimmed = raw.trim();
  if (trimmed === "") return { valid: false };
  const num = Number(trimmed);
  if (!Number.isInteger(num) || !Number.isFinite(num)) return { valid: false };
  return { valid: true, value: num };
}
