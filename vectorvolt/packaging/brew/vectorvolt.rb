class Vectorvolt < Formula
  desc "VectorVolt — ultra-low-latency kernel-bypass data engine (vvctl CLI)"
  homepage "https://workerruntime.com"
  version "0.1.0"
  license "Apache-2.0"

  on_macos do
    on_arm do
      url "https://github.com/workerruntime/vectorvolt/releases/download/v#{version}/vvctl-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    end
    on_intel do
      url "https://github.com/workerruntime/vectorvolt/releases/download/v#{version}/vvctl-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/workerruntime/vectorvolt/releases/download/v#{version}/vvctl-v#{version}-aarch64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER_LINUX_ARM64_SHA256"
    end
    on_intel do
      url "https://github.com/workerruntime/vectorvolt/releases/download/v#{version}/vvctl-v#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER_LINUX_X86_64_SHA256"
    end
  end

  def install
    bin.install "vvctl"
  end

  test do
    system "#{bin}/vvctl", "version"
  end
end
