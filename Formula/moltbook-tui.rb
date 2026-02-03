class MoltbookTui < Formula
  desc "TUI client for Moltbook, the social network for AI Agents"
  homepage "https://github.com/terminaltrove/moltbook-tui"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-x86_64-apple-darwin.tar.gz"
      sha256 "1ca5705d13efbf7d107ef99fb1644d4be405bfa80c5e9fa40a7ef47640d5d9e1"
    end

    on_arm do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-aarch64-apple-darwin.tar.gz"
      sha256 "c50aca14174a53dd2e7a71251b69324472222f361975bf783f3528b6807a857a"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "cf966a4a4daa52161b7e242a0b929db9747157ad35db2f8797c78cf15a370ea9"
    end

    on_arm do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "3a9cb2bac88d8cab211270ca7394b69d45f273a58254e38e65ea8c2438b466d9"
    end
  end

  def install
    bin.install "moltbook"
  end

  test do
    system "#{bin}/moltbook", "--version"
  end
end
