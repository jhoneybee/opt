Set oArgs = WScript.Arguments
    Set wrd = GetObject("", "Word.Application")
        wrd.Visible = false
        path = createobject("Scripting.FileSystemObject").GetFolder(".").Path
        wrd.Documents.Open path + "/" + oArgs(0)
        wrd.ActiveDocument.SaveAs2 path + "/" + oArgs(1), 17
        wrd.Quit
    Set wrd = Nothing
Set oArgs = Nothing