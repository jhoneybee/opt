Set oArgs = WScript.Arguments
    IF oArgs(2) = "WORD" Then 
        Set wrd = GetObject("", "Word.Application")
            wrd.Visible = false
            wrd.Documents.Open oArgs(0)
            wrd.ActiveDocument.SaveAs2 oArgs(1), 17
            wrd.Quit
        Set wrd = Nothing
    ElseIf oArgs(2) = "EXCEL" Then
        Set excel = GetObject("", "Excel.Application")
            excel.Visible = False
            excel.Workbooks.Open oArgs(0)
            excel.ActiveSheet.ExportAsFixedFormat 0, oArgs(1)
            excel.Quit
        Set excel = Nothing
    ElseIf oArgs(2) = "PPT" Then
        Set msppt = GetObject("", "Powerpoint.Application")
            msppt.Presentations.Open oArgs(0),,,true
            msppt.ActivePresentation.SaveAs oArgs(1), 32
            msppt.Quit
        Set msppt = Nothing
    End If
Set oArgs = Nothing