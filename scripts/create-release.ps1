# Oxide Pilot - Release Creation Script
# This script helps create releases manually

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet('stable', 'dev', 'custom')]
    [string]$Type = 'dev',
    
    [Parameter(Mandatory=$false)]
    [string]$Version,
    
    [Parameter(Mandatory=$false)]
    [string]$CustomTag,
    
    [switch]$DryRun
)

Write-Host "🚀 Oxide Pilot Release Creator" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Check if we're in a git repository
if (-not (Test-Path .git)) {
    Write-Host "❌ Error: Not in a git repository" -ForegroundColor Red
    exit 1
}

# Check for uncommitted changes
$status = git status --porcelain
if ($status -and -not $DryRun) {
    Write-Host "⚠️  Warning: You have uncommitted changes:" -ForegroundColor Yellow
    Write-Host $status
    $continue = Read-Host "Continue anyway? (y/N)"
    if ($continue -ne 'y') {
        Write-Host "Aborted." -ForegroundColor Yellow
        exit 0
    }
}

# Get current branch
$branch = git rev-parse --abbrev-ref HEAD
Write-Host "📍 Current branch: $branch" -ForegroundColor Green

# Get current version from Cargo.toml
$cargoToml = Get-Content src-tauri/Cargo.toml -Raw
$currentVersion = ($cargoToml | Select-String 'version = "(.+)"').Matches.Groups[1].Value
Write-Host "📦 Current version in Cargo.toml: $currentVersion" -ForegroundColor Green

switch ($Type) {
    'stable' {
        Write-Host ""
        Write-Host "🏷️  Creating STABLE release" -ForegroundColor Cyan
        
        if (-not $Version) {
            Write-Host ""
            Write-Host "Current version: $currentVersion"
            $Version = Read-Host "Enter new version (e.g., 1.0.0)"
            
            if (-not $Version) {
                Write-Host "❌ Version is required for stable releases" -ForegroundColor Red
                exit 1
            }
        }
        
        # Validate semantic versioning
        if ($Version -notmatch '^\d+\.\d+\.\d+$') {
            Write-Host "❌ Invalid version format. Use semantic versioning (e.g., 1.0.0)" -ForegroundColor Red
            exit 1
        }
        
        $tag = "v$Version"
        
        Write-Host ""
        Write-Host "📝 Steps to execute:" -ForegroundColor Yellow
        Write-Host "  1. Update version in src-tauri/Cargo.toml to $Version"
        Write-Host "  2. Commit the version change"
        Write-Host "  3. Create tag: $tag"
        Write-Host "  4. Push tag to trigger release workflow"
        
        if (-not $DryRun) {
            Write-Host ""
            $confirm = Read-Host "Proceed? (y/N)"
            if ($confirm -ne 'y') {
                Write-Host "Aborted." -ForegroundColor Yellow
                exit 0
            }
            
            # Update version in Cargo.toml
            Write-Host ""
            Write-Host "📝 Updating Cargo.toml..." -ForegroundColor Cyan
            $cargoToml = $cargoToml -replace 'version = ".+"', "version = `"$Version`""
            Set-Content src-tauri/Cargo.toml -Value $cargoToml
            
            # Commit
            git add src-tauri/Cargo.toml
            git commit -m "chore: bump version to $Version"
            
            # Create tag
            Write-Host "🏷️  Creating tag $tag..." -ForegroundColor Cyan
            git tag $tag
            
            # Push
            Write-Host "⬆️  Pushing to origin..." -ForegroundColor Cyan
            git push origin $branch
            git push origin $tag
            
            Write-Host ""
            Write-Host "✅ Release $tag created successfully!" -ForegroundColor Green
            Write-Host "🔗 Check progress at: https://github.com/yourusername/oxide-pilot/actions" -ForegroundColor Cyan
        } else {
            Write-Host ""
            Write-Host "🔍 DRY RUN - No changes made" -ForegroundColor Yellow
        }
    }
    
    'dev' {
        Write-Host ""
        Write-Host "🧪 Creating DEVELOPMENT release" -ForegroundColor Cyan
        Write-Host "This will trigger an automatic bootstrap release when pushed to main" -ForegroundColor Gray
        
        Write-Host ""
        Write-Host "📝 Steps to execute:" -ForegroundColor Yellow
        Write-Host "  1. Push current changes to main branch"
        Write-Host "  2. Automatic bootstrap tag will be created: bootstrap-TIMESTAMP-COMMIT"
        Write-Host "  3. Pre-release will be created automatically"
        
        if (-not $DryRun) {
            Write-Host ""
            $confirm = Read-Host "Push to main now? (y/N)"
            if ($confirm -ne 'y') {
                Write-Host "Aborted." -ForegroundColor Yellow
                exit 0
            }
            
            if ($branch -ne 'main') {
                Write-Host "⚠️  You are not on main branch. Switch to main first." -ForegroundColor Yellow
                $switch = Read-Host "Switch to main? (y/N)"
                if ($switch -eq 'y') {
                    git checkout main
                    git pull origin main
                } else {
                    Write-Host "Aborted." -ForegroundColor Yellow
                    exit 0
                }
            }
            
            Write-Host "⬆️  Pushing to main..." -ForegroundColor Cyan
            git push origin main
            
            Write-Host ""
            Write-Host "✅ Push completed! Automatic release will be created." -ForegroundColor Green
            Write-Host "🔗 Check progress at: https://github.com/yourusername/oxide-pilot/actions" -ForegroundColor Cyan
        } else {
            Write-Host ""
            Write-Host "🔍 DRY RUN - No changes made" -ForegroundColor Yellow
        }
    }
    
    'custom' {
        Write-Host ""
        Write-Host "🎨 Creating CUSTOM release" -ForegroundColor Cyan
        
        if (-not $CustomTag) {
            $CustomTag = Read-Host "Enter custom tag name (e.g., bootstrap-feature-xyz)"
            
            if (-not $CustomTag) {
                Write-Host "❌ Custom tag is required" -ForegroundColor Red
                exit 1
            }
        }
        
        Write-Host ""
        Write-Host "📝 Steps to execute:" -ForegroundColor Yellow
        Write-Host "  1. Create tag: $CustomTag"
        Write-Host "  2. Push tag to trigger release workflow"
        
        if (-not $DryRun) {
            Write-Host ""
            $confirm = Read-Host "Proceed? (y/N)"
            if ($confirm -ne 'y') {
                Write-Host "Aborted." -ForegroundColor Yellow
                exit 0
            }
            
            Write-Host "🏷️  Creating tag $CustomTag..." -ForegroundColor Cyan
            git tag $CustomTag
            
            Write-Host "⬆️  Pushing tag..." -ForegroundColor Cyan
            git push origin $CustomTag
            
            Write-Host ""
            Write-Host "✅ Release $CustomTag created successfully!" -ForegroundColor Green
            Write-Host "🔗 Check progress at: https://github.com/yourusername/oxide-pilot/actions" -ForegroundColor Cyan
        } else {
            Write-Host ""
            Write-Host "🔍 DRY RUN - No changes made" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "📚 For more information, see .github/RELEASE_AUTOMATION.md" -ForegroundColor Gray
