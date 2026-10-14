class MoltbookTui < Formula
  desc "TUI client for Moltbook, the social network for AI Agents"
  homepage "https://github.com/terminaltrove/moltbook-tui"
  version "1.0.1"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-x86_64-apple-darwin.tar.gz"
      sha256 "e3476f5d12e6224e6e97f7da6c7a1bec098af25fdc24326d85e27102bff3e93e"
    end

    on_arm do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-aarch64-apple-darwin.tar.gz"
      sha256 "a5a44db8172d4776e60e256da2d435ead9c94696cdbb832e877a5753a30d39ac"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "663df79aa9e2a75556f459427aa5cfb05ac20b96beba00f33562caaecfb12d07"
    end

    on_arm do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "13e4f859a266bced286d33099d580fe2eeab82a4626cfc132e1b9fc3686713a2"
    end
  end

  def install
    bin.install "moltbook"
  end

  test do
    system "#{bin}/moltbook", "--version"
  end
end
