from dash import Dash, html
import dash_cytoscape as cyto
import pandas as pd
app = Dash(__name__)

df = pd.read_csv("./out.csv")
data = [{"data": {"source": row["hash"], "target": f'{row["url"]}({row["code"]})'}} for _, row in df.iterrows()]
edges_source = [{"data": {"id": row["hash"], "label": row["hash"]}} for _, row in df.iterrows()]
edges_target = [{"data": {"id": f'{row["url"]}({row["code"]})', "label": f'{row["url"]}({row["code"]})'}}
    for _, row in df.iterrows()]
result = edges_source + edges_target + data

app.layout = html.Div([
    html.P("Scan results"),
    cyto.Cytoscape(
        id='cytoscape',
        elements=result, 
        layout={'name': 'breadthfirst'},
        style={'width': '1200px', 'height': '800px'}
    )
])


app.run_server(debug=True)
