param(
    [Parameter(Mandatory = $true)]
    [string]$DirectoryPath
)

Get-ChildItem -LiteralPath $DirectoryPath -ErrorAction Stop |
    Select-Object Name, Mode