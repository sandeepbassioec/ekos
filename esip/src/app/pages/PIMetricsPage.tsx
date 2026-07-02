import React, { useEffect, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router';
import { ArrowLeft, TrendingUp, Zap, CheckCircle2, BarChart2 } from 'lucide-react';
import {
  BarChart, Bar, LineChart, Line, XAxis, YAxis, CartesianGrid,
  Tooltip, Legend, ResponsiveContainer, ReferenceLine,
} from 'recharts';
import { usePIStore } from '../store/piStore';
import { useRequirementsStore } from '../store/requirementsStore';
import type { Sprint, Requirement } from '../lib/db/types';

// ── Helpers ───────────────────────────────────────────────────────────────────

function isDone(r: Requirement) {
  return r.status === 'done' || r.status === 'completed' || r.status === 'finalized';
}

function shortDate(d?: string) {
  if (!d) return '';
  return new Date(d).toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });
}

// ── Metric card ───────────────────────────────────────────────────────────────

function MetricCard({ label, value, sub, icon, color }: {
  label: string; value: string | number; sub?: string;
  icon: React.ReactNode; color: string;
}) {
  return (
    <div className={`flex items-center gap-4 bg-white rounded-xl border border-gray-200 shadow-sm px-5 py-4`}>
      <div className={`w-10 h-10 rounded-full flex items-center justify-center ${color}`}>
        {icon}
      </div>
      <div>
        <p className="text-xs text-gray-500">{label}</p>
        <p className="text-2xl font-bold text-gray-900">{value}</p>
        {sub && <p className="text-xs text-gray-400">{sub}</p>}
      </div>
    </div>
  );
}

// ── PIMetricsPage ─────────────────────────────────────────────────────────────

export default function PIMetricsPage() {
  const { projectId, piId } = useParams<{ projectId: string; piId: string }>();
  const navigate = useNavigate();

  const { pis, sprints: allSprints, fetchPIs } = usePIStore();
  const { requirements, fetchRequirements } = useRequirementsStore();

  useEffect(() => {
    if (!projectId) return;
    fetchPIs(projectId);
    fetchRequirements(projectId);
  }, [projectId]);

  const pi = piId ? pis.find(p => p.id === piId) : pis[0];
  const sprints = useMemo(
    () => allSprints
      .filter(s => s.piId === (pi?.id ?? ''))
      .sort((a, b) => (a.startDate ?? '').localeCompare(b.startDate ?? '')),
    [allSprints, pi],
  );

  const piReqs = requirements.filter(r => r.piId === pi?.id);

  // ── Velocity data: completed reqs per sprint ──────────────────────────────
  const velocityData = useMemo(() => sprints.map(s => {
    const sprintReqs = requirements.filter(r => r.sprintId === s.id);
    return {
      name: s.name,
      label: shortDate(s.startDate),
      completed: sprintReqs.filter(isDone).length,
      total: sprintReqs.length,
    };
  }), [sprints, requirements]);

  const avgVelocity = velocityData.length
    ? Math.round(velocityData.reduce((sum, d) => sum + d.completed, 0) / velocityData.length)
    : 0;

  // ── Burn-up data: cumulative scope vs cumulative done over sprints ──────────
  const burnUpData = useMemo(() => {
    let cumDone = 0;
    return sprints.map((s, i) => {
      const sprintReqs = requirements.filter(r => r.sprintId === s.id);
      cumDone += sprintReqs.filter(isDone).length;
      // scope = total requirements allocated to sprints up to and including this one
      const scope = sprints.slice(0, i + 1).reduce(
        (sum, sp) => sum + requirements.filter(r => r.sprintId === sp.id).length,
        0,
      );
      return {
        name: s.name,
        label: shortDate(s.endDate),
        scope,
        done: cumDone,
      };
    });
  }, [sprints, requirements]);

  // ── Summary stats ──────────────────────────────────────────────────────────
  const totalAllocated = piReqs.length;
  const totalDone = piReqs.filter(isDone).length;
  const completedSprints = sprints.filter(s => s.status === 'completed').length;
  const pct = totalAllocated > 0 ? Math.round((totalDone / totalAllocated) * 100) : 0;

  if (pis.length === 0) {
    return (
      <div className="flex flex-col gap-6 p-6">
        <button
          onClick={() => navigate(`/projects/${projectId}/pi-planning`)}
          className="flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-800"
        >
          <ArrowLeft className="w-4 h-4" /> PI Planning
        </button>
        <div className="flex flex-col items-center justify-center py-20 gap-3 text-center">
          <BarChart2 className="w-12 h-12 text-gray-300" />
          <p className="text-gray-500 font-medium">No PIs yet — create one in PI Planning first.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 p-6 bg-gray-50 min-h-full">
      {/* Nav */}
      <button
        onClick={() => navigate(`/projects/${projectId}/pi-planning`)}
        className="flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-800 w-fit"
      >
        <ArrowLeft className="w-4 h-4" /> PI Planning
      </button>

      {/* Header */}
      <div className="flex items-start justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Metrics & Velocity</h1>
          <p className="text-sm text-gray-500 mt-0.5">{pi?.name} · {shortDate(pi?.startDate)} – {shortDate(pi?.endDate)}</p>
        </div>

        {/* PI selector if multiple PIs */}
        {pis.length > 1 && (
          <select
            value={pi?.id ?? ''}
            onChange={e => navigate(`/projects/${projectId}/pi-planning/metrics/${e.target.value}`)}
            className="text-sm border border-gray-200 rounded-lg px-3 py-1.5 bg-white shadow-sm"
          >
            {pis.map(p => (
              <option key={p.id} value={p.id}>{p.name}</option>
            ))}
          </select>
        )}
      </div>

      {/* Summary cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          label="Requirements Allocated"
          value={totalAllocated}
          icon={<BarChart2 className="w-5 h-5 text-indigo-600" />}
          color="bg-indigo-100"
        />
        <MetricCard
          label="Completed"
          value={totalDone}
          sub={`${pct}% of allocated`}
          icon={<CheckCircle2 className="w-5 h-5 text-green-600" />}
          color="bg-green-100"
        />
        <MetricCard
          label="Avg Velocity"
          value={avgVelocity}
          sub="reqs/sprint"
          icon={<Zap className="w-5 h-5 text-amber-600" />}
          color="bg-amber-100"
        />
        <MetricCard
          label="Sprints Completed"
          value={`${completedSprints} / ${sprints.length}`}
          icon={<TrendingUp className="w-5 h-5 text-blue-600" />}
          color="bg-blue-100"
        />
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        {/* Velocity chart */}
        <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-5">
          <h2 className="text-base font-semibold text-gray-800 mb-1">Sprint Velocity</h2>
          <p className="text-xs text-gray-400 mb-4">Requirements completed per sprint</p>
          {velocityData.every(d => d.total === 0) ? (
            <div className="flex items-center justify-center h-48 text-gray-400 text-sm">
              No requirements allocated to sprints yet
            </div>
          ) : (
            <ResponsiveContainer width="100%" height={220}>
              <BarChart data={velocityData} margin={{ top: 4, right: 8, bottom: 4, left: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
                <XAxis dataKey="name" tick={{ fontSize: 11 }} />
                <YAxis allowDecimals={false} tick={{ fontSize: 11 }} />
                <Tooltip
                  formatter={(val, name) => [val, name === 'completed' ? 'Completed' : 'Total']}
                  labelFormatter={l => `${l}`}
                />
                <Legend formatter={v => v === 'completed' ? 'Completed' : 'Total Allocated'} />
                <ReferenceLine y={avgVelocity} stroke="#6366f1" strokeDasharray="4 2" label={{ value: `Avg ${avgVelocity}`, position: 'insideTopRight', fontSize: 10, fill: '#6366f1' }} />
                <Bar dataKey="total" fill="#e0e7ff" radius={[4, 4, 0, 0]} name="total" />
                <Bar dataKey="completed" fill="#6366f1" radius={[4, 4, 0, 0]} name="completed" />
              </BarChart>
            </ResponsiveContainer>
          )}
        </div>

        {/* Burn-up chart */}
        <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-5">
          <h2 className="text-base font-semibold text-gray-800 mb-1">Burn-up Chart</h2>
          <p className="text-xs text-gray-400 mb-4">Cumulative scope vs completed over time</p>
          {burnUpData.every(d => d.scope === 0) ? (
            <div className="flex items-center justify-center h-48 text-gray-400 text-sm">
              No requirements allocated to sprints yet
            </div>
          ) : (
            <ResponsiveContainer width="100%" height={220}>
              <LineChart data={burnUpData} margin={{ top: 4, right: 8, bottom: 4, left: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
                <XAxis dataKey="name" tick={{ fontSize: 11 }} />
                <YAxis allowDecimals={false} tick={{ fontSize: 11 }} />
                <Tooltip />
                <Legend formatter={v => v === 'scope' ? 'Scope (Total)' : 'Completed'} />
                <Line type="monotone" dataKey="scope" stroke="#94a3b8" strokeWidth={2} dot={false} name="scope" />
                <Line type="monotone" dataKey="done" stroke="#22c55e" strokeWidth={2} dot={{ r: 3, fill: '#22c55e' }} name="done" />
              </LineChart>
            </ResponsiveContainer>
          )}
        </div>
      </div>

      {/* Sprint breakdown table */}
      <div className="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden">
        <div className="px-5 py-3 border-b border-gray-100">
          <h2 className="text-base font-semibold text-gray-800">Sprint Breakdown</h2>
        </div>
        <table className="w-full text-sm">
          <thead className="bg-gray-50 text-xs text-gray-500 uppercase tracking-wide">
            <tr>
              <th className="px-5 py-2.5 text-left">Sprint</th>
              <th className="px-5 py-2.5 text-left">Dates</th>
              <th className="px-5 py-2.5 text-right">Allocated</th>
              <th className="px-5 py-2.5 text-right">Completed</th>
              <th className="px-5 py-2.5 text-right">Velocity</th>
              <th className="px-5 py-2.5 text-right">Progress</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-50">
            {sprints.map(s => {
              const sr = requirements.filter(r => r.sprintId === s.id);
              const done = sr.filter(isDone).length;
              const total = sr.length;
              const p = total > 0 ? Math.round((done / total) * 100) : 0;
              return (
                <tr key={s.id} className="hover:bg-gray-50/50 transition-colors">
                  <td className="px-5 py-3 font-medium text-gray-800">{s.name}</td>
                  <td className="px-5 py-3 text-gray-500 text-xs">{shortDate(s.startDate)} – {shortDate(s.endDate)}</td>
                  <td className="px-5 py-3 text-right text-gray-700">{total}</td>
                  <td className="px-5 py-3 text-right text-gray-700">{done}</td>
                  <td className="px-5 py-3 text-right">
                    <span className={`font-semibold ${done >= avgVelocity ? 'text-green-600' : 'text-gray-400'}`}>{done}</span>
                  </td>
                  <td className="px-5 py-3 text-right">
                    <div className="flex items-center justify-end gap-2">
                      <div className="w-16 h-1.5 bg-gray-100 rounded-full overflow-hidden">
                        <div
                          className={`h-full rounded-full ${p >= 90 ? 'bg-green-500' : p >= 50 ? 'bg-indigo-500' : 'bg-amber-400'}`}
                          style={{ width: `${p}%` }}
                        />
                      </div>
                      <span className="text-xs text-gray-500 w-8 text-right">{p}%</span>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
