import { formatDateTime } from '@/lib/helpers';

type ProbeResult = {
  probe_name: string;
  timestamp_started: string;
  success: boolean;
  error_message?: string;
  trace_id?: string;
};

type ProbeResultItemProps = {
  result: ProbeResult;
};

export default function ProbeResultItem({ result }: ProbeResultItemProps) {
  return (
    <div className="border-b border-gray-200 last:border-0 p-4 hover:bg-gray-50">
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
      
      {result.trace_id && (
        <div className="text-xs text-gray-500 font-mono">
          Trace ID: {result.trace_id}
        </div>
      )}
      
      {result.error_message && (
        <div className="mt-2 text-sm text-red-600 bg-red-50 p-2 rounded">
          {result.error_message}
        </div>
      )}
    </div>
  );
}