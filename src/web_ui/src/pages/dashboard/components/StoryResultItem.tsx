import { useState } from 'react';
import { formatDateTime } from '@/lib/helpers';

type StepResult = {
  step_name: string;
  timestamp_started: string;
  success: boolean;
  error_message?: string;
  trace_id?: string;
  span_id?: string;
};

type StoryResult = {
  story_name: string;
  timestamp_started: string;
  success: boolean;
  step_results: StepResult[];
};

type StoryResultItemProps = {
  result: StoryResult;
};

export default function StoryResultItem({ result }: StoryResultItemProps) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="border-b border-gray-200 last:border-0 p-4">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm text-gray-600">
          {formatDateTime(result.timestamp_started)}
        </span>
        <span
          className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
            result.success
              ? 'bg-green-100 text-green-800'
              : 'bg-red-100 text-red-800'
          }`}
        >
          {result.success ? 'Success' : 'Failed'}
        </span>
      </div>

      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-500">
          {result.step_results.length} steps
        </span>
        <button
          onClick={() => setExpanded(!expanded)}
          className="text-xs text-blue-600 hover:text-blue-800 font-medium cursor-pointer"
        >
          {expanded ? 'Hide Steps' : 'Show Steps'}
        </button>
      </div>

      {expanded && (
        <div className="mt-3 space-y-2">
          {result.step_results.map((step, index) => (
            <div
              key={`${step.step_name}-${index}`}
              className="bg-gray-50 p-2 rounded text-xs"
            >
              <div className="flex items-center gap-2 mb-1">
                <span
                  className={`inline-flex items-center px-1.5 py-0.5 rounded text-xs ${
                    step.success
                      ? 'bg-green-200 text-green-800'
                      : 'bg-red-200 text-red-800'
                  }`}
                >
                  {step.success ? '✓' : '✗'}
                </span>
                <span className="font-medium text-gray-700">
                  {step.step_name}
                </span>
              </div>

              <div className="text-gray-500 mb-1">
                {formatDateTime(step.timestamp_started)}
              </div>

              {step.trace_id && (
                <div className="text-gray-500 font-mono mb-1">
                  Trace: {step.trace_id}
                </div>
              )}

              {step.error_message && (
                <div className="text-red-600 bg-red-100 p-1 rounded">
                  {step.error_message}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
