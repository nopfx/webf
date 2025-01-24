from dash import Dash, html
import dash_cytoscape as cyto
import pandas as pd
import sys
app = Dash(__name__)

df = pd.read_csv(sys.argv[1])
data = [
    {
        "data": {
            "source": f'{row["hash"]}({row["bytes"]})',
            "target": f'{row["url"]}({row["code"]})'
        }
    }
    for _, row in df.iterrows()
]

edges_source = [{"data": {"id": f'{row["hash"]}({row["bytes"]})', "label": f'{row["hash"]}({row["bytes"]})'},"classes":"green pages"} for _, row in df.iterrows()]

edges_target = [
    {
        "data": {
            "id": f'{row["url"]}({row["code"]})',
            "label": f'{row["url"]}({row["code"]})',
        },
        "classes": "red" if row["code"] == 404 or row["code"] == 403 else "yellow" if row["code"] == 401 else "purple" if row["code"] == 500 else "green" if row["code"] == 200 else "default",
    }
    for _, row in df.iterrows()]


result = edges_source + edges_target + data

app.layout = html.Div([
    html.P("Scan results"),
    cyto.Cytoscape(
        id='cytoscape',
        elements=result, 
        layout={'name': 'breadthfirst', 'circle':True},
        style={'width': '100%', 'height': '800px'},
        stylesheet=[
            {
                'selector':'node',
                'style': {
                    'content':'data(label)',
                    'width':18,
                    'height':18,

                }
            },
            {
                'selector':'.green',
                'style':{
                    'background-color':'#0E9AA7',
                }
            },
            {
                'selector':'.red',
                'style':{
                    'background-color':'#FE8A71',
                }
            },
            {
                'selector':'.yellow',
                'style':{
                    'background-color':'#F6CD61',
                }
            },
            {
                'selector':'.pages',
                'style':{
                    'shape':'rectangle',
                    'background-color':'#4a4e4d',
                    "width":24,
                    "height":24,
                }
            },
            {
                'selector':'.purple',
                'style':{
                    'background-color':'#EDBEE4',
                }
            }
        ]
    )
])


app.run_server(debug=True)
