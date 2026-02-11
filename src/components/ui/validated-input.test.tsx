import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ValidatedInput } from "./validated-input";
import { validatePort, validateMidiNote } from "@/lib/validators";

// Mock sonner toast
vi.mock("sonner", () => ({
  toast: { error: vi.fn() },
}));

import { toast } from "sonner";

describe("ValidatedInput", () => {
  it("renders with initial value", () => {
    render(
      <ValidatedInput
        value={8080}
        onCommit={() => {}}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    expect(screen.getByDisplayValue("8080")).toBeDefined();
  });

  it("allows free typing without clamping", () => {
    render(
      <ValidatedInput
        value={8080}
        onCommit={() => {}}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    const input = screen.getByDisplayValue("8080") as HTMLInputElement;
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "8" } });
    expect(input.value).toBe("8");
  });

  it("commits valid value on blur", () => {
    const onCommit = vi.fn();
    render(
      <ValidatedInput
        value={8080}
        onCommit={onCommit}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    const input = screen.getByDisplayValue("8080");
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "9090" } });
    fireEvent.blur(input);
    expect(onCommit).toHaveBeenCalledWith(9090);
  });

  it("reverts and shows toast on invalid blur", () => {
    const onCommit = vi.fn();
    render(
      <ValidatedInput
        value={8080}
        onCommit={onCommit}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    const input = screen.getByDisplayValue("8080") as HTMLInputElement;
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "abc" } });
    fireEvent.blur(input);
    expect(onCommit).not.toHaveBeenCalled();
    expect(toast.error).toHaveBeenCalledWith("Invalid port");
    expect(input.value).toBe("8080");
  });

  it("shows red border for invalid text", () => {
    render(
      <ValidatedInput
        value={8080}
        onCommit={() => {}}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    const input = screen.getByDisplayValue("8080");
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "abc" } });
    expect(input.className).toContain("border-red-500");
  });

  it("accepts note names for MIDI note validation", () => {
    const onCommit = vi.fn();
    render(
      <ValidatedInput
        value={60}
        onCommit={onCommit}
        validate={validateMidiNote}
        errorMessage="Invalid note"
      />
    );
    const input = screen.getByDisplayValue("60");
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "C3" } });
    fireEvent.blur(input);
    expect(onCommit).toHaveBeenCalledWith(60);
  });

  it("syncs from parent when not focused", () => {
    const { rerender } = render(
      <ValidatedInput
        value={8080}
        onCommit={() => {}}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    rerender(
      <ValidatedInput
        value={9090}
        onCommit={() => {}}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    expect(screen.getByDisplayValue("9090")).toBeDefined();
  });

  it("does not sync from parent when focused", () => {
    const { rerender } = render(
      <ValidatedInput
        value={8080}
        onCommit={() => {}}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    const input = screen.getByDisplayValue("8080") as HTMLInputElement;
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "1234" } });
    rerender(
      <ValidatedInput
        value={9090}
        onCommit={() => {}}
        validate={validatePort}
        errorMessage="Invalid port"
      />
    );
    expect(input.value).toBe("1234");
  });
});
