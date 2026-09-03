import React from "react";

function renderLine(line: string, key: number): React.ReactNode {
  const trimmed = line.trimEnd();
  if (!trimmed) return <div key={key} className="h-2" />;
  if (trimmed.startsWith("### "))
    return (
      <h3 key={key} className="text-sm font-semibold mt-3 mb-1 text-neutral-800">
        {trimmed.slice(4)}
      </h3>
    );
  if (trimmed.startsWith("## "))
    return (
      <h2 key={key} className="text-base font-bold mt-4 mb-2 text-neutral-900">
        {trimmed.slice(3)}
      </h2>
    );
  if (trimmed.startsWith("> "))
    return (
      <blockquote
        key={key}
        className="border-l-2 border-primary pl-3 my-2 text-xs text-neutral-500"
      >
        {trimmed.slice(2)}
      </blockquote>
    );
  if (trimmed.startsWith("---"))
    return <hr key={key} className="my-3 border-neutral-200" />;
  if (trimmed.startsWith("- "))
    return (
      <li key={key} className="ml-5 list-disc text-sm text-neutral-700">
        {trimmed.slice(2)}
      </li>
    );
  return (
    <p key={key} className="text-sm text-neutral-700 leading-relaxed">
      {trimmed}
    </p>
  );
}

export default function Markdown({ content }: { content: string }) {
  const lines = content.split("\n");
  return (
    <div className="prose-sm">
      {lines.map((l, i) => renderLine(l, i))}
    </div>
  );
}
