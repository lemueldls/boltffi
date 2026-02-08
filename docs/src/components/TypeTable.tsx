import { useState } from "react";
import { cn } from "@/lib/utils";

interface TypeMapping {
  rust: string;
  swift: string;
  kotlin: string;
}

interface TypeTableProps {
  title: string;
  mappings: TypeMapping[];
}

const TypeTable = ({ title, mappings }: TypeTableProps) => {
  const [activeLang, setActiveLang] = useState<"Swift" | "Kotlin">("Swift");

  return (
    <div className="my-3">
      <div className="flex items-center gap-3 mb-2">
        <span className="text-sm font-medium text-muted-foreground">{title}</span>
        <div className="flex items-center gap-0.5">
          {(["Swift", "Kotlin"] as const).map((lang) => (
            <button
              key={lang}
              onClick={() => setActiveLang(lang)}
              className={cn(
                "px-2 py-0.5 rounded text-xs font-mono transition-colors",
                activeLang === lang
                  ? "bg-primary/20 text-primary"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              {lang}
            </button>
          ))}
        </div>
      </div>
      <div className="flex flex-wrap gap-x-6 gap-y-1">
        {mappings.map((mapping, i) => (
          <div key={i} className="flex items-center gap-2">
            <code className="text-sm font-mono text-primary">{mapping.rust}</code>
            <span className="text-muted-foreground text-sm">→</span>
            <code className="text-sm font-mono text-muted-foreground">
              {activeLang === "Swift" ? mapping.swift : mapping.kotlin}
            </code>
          </div>
        ))}
      </div>
    </div>
  );
};

export default TypeTable;
