import { useState, useEffect, useRef, useCallback } from "react";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import type { ValidationResult } from "@/lib/validators";

interface ValidatedInputProps {
  value: number | string;
  onCommit: (value: number) => void;
  validate: (raw: string) => ValidationResult;
  errorMessage: string;
  className?: string;
  disabled?: boolean;
  placeholder?: string;
  inputMode?: "numeric" | "text" | "decimal";
}

export function ValidatedInput({
  value,
  onCommit,
  validate,
  errorMessage,
  className,
  disabled,
  placeholder,
  inputMode,
}: ValidatedInputProps) {
  const [text, setText] = useState(String(value));
  const focused = useRef(false);

  // Sync from parent when not focused
  useEffect(() => {
    if (!focused.current) {
      setText(String(value));
    }
  }, [value]);

  const isInvalid = text.trim() !== "" && !validate(text).valid;

  const handleBlur = useCallback(() => {
    focused.current = false;
    const result = validate(text);
    if (result.valid) {
      onCommit(result.value);
    } else {
      toast.error(errorMessage);
      setText(String(value));
    }
  }, [text, validate, onCommit, errorMessage, value]);

  return (
    <Input
      className={`${className ?? ""} ${isInvalid ? "border-red-500" : ""}`}
      value={text}
      disabled={disabled}
      placeholder={placeholder}
      inputMode={inputMode}
      onFocus={() => {
        focused.current = true;
      }}
      onChange={(e) => setText(e.target.value)}
      onBlur={handleBlur}
    />
  );
}
