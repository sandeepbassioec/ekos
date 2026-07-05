Name:       vectorvolt
Version:    0.1.0
Release:    1%{?dist}
Summary:    VectorVolt — ultra-low-latency kernel-bypass data engine
License:    Apache-2.0
URL:        https://workerruntime.com
Source0:    vvctl-%{version}-x86_64-unknown-linux-musl.tar.gz

%description
VectorVolt is a zero-GC, kernel-bypass Write-Ahead Log and storage engine
designed for sub-microsecond latency workloads. This package installs vvctl,
the VectorVolt control plane CLI.

%prep
%setup -q -n vvctl-%{version}-x86_64-unknown-linux-musl

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 vvctl %{buildroot}%{_bindir}/vvctl

%files
%{_bindir}/vvctl

%changelog
* Sat Jul 05 2026 WorkerRuntime <hello@workerruntime.com> - 0.1.0-1
- Phase 1: WAL foundation, GroupCommitBatcher, vvctl CLI
