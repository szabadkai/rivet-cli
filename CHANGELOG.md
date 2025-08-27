# Changelog

All notable changes to Rivet CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 🎨 New **compact HTML template** with interactive features:
  - Real-time search and filtering (All/Passed/Failed tests)
  - Expandable test details with explanations
  - Light/dark theme toggle with persistent preferences
  - Mobile-responsive design
- ⚙️ **User configuration system** (`~/.rivet/config.json`):
  - Configurable auto-open browser behavior (default: true)
  - Configurable default HTML template (default: compact)
  - Automatic config file creation with sensible defaults
- 🎯 **Enhanced HTML templates**:
  - `detailed` - Professional business reports
  - `compact` - Interactive, filterable interface (NEW)
  - `chatty` - Friendly, conversational style
  - `simple` - Minimal, clean reports
- 🚀 **Auto-open browser functionality**:
  - `--open` flag to force browser opening
  - `--no-open` flag to disable browser opening
  - Respects user configuration by default
- 🔧 **GitHub Actions CI/CD**:
  - Automated building for Linux, macOS (Intel/ARM), and Windows
  - Automated releases with pre-built binaries
  - Continuous integration with tests and linting
  - Automated dependency updates via Dependabot

### Enhanced
- 📊 Improved HTML report generation with better error handling
- 🎨 Enhanced visual design across all templates
- 🔍 Better test result filtering and organization
- ⚡ Performance optimizations in report generation

### Fixed
- 🐛 Template parsing errors with complex Tera syntax
- 📱 Mobile responsiveness across all HTML templates
- 🎨 Theme consistency and color scheme improvements

## [0.1.0] - Initial Release

### Added
- ✨ Core HTTP request functionality
- 🧪 Test suite execution with YAML configuration
- 📊 Basic HTML, JSON, and JUnit report generation
- 📥 Import support for Postman, Insomnia, Bruno, and cURL
- 🔗 gRPC unary call support
- 📝 OpenAPI spec integration and coverage reporting
- 🎨 Beautiful terminal UI with progress indicators
- 🚀 Parallel test execution
- 📊 Data-driven testing with CSV datasets
- 🌍 Environment variable support