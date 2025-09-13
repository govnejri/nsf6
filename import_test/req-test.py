import requests
import json

# Параметры запроса
url = 'http://127.0.0.1:8080/api/heatmap'
params = {
    'tlLat': '51.20',
    'tlLong': '71.30',
    'brLat': '51.00',
    'brLong': '71.60',
    'timeStart': '2025-09-01T00:00:00Z',
    'timeEnd': '2025-09-14T00:00:00Z',
    'tileWidth': '0.01',
    'tileHeight': '0.01'
}

try:
    # Выполнение GET-запроса
    response = requests.get(url, params=params)
    response.raise_for_status()  # Проверка на ошибки HTTP
    # Парсинг JSON-ответа. Some APIs return a JSON string or wrap the list.
    data = response.json()

    # If server returned a JSON string, try to parse it again
    if isinstance(data, str):
        try:
            data = json.loads(data)
        except json.JSONDecodeError:
            raise ValueError("Response JSON is a string but not valid JSON list/dict")

    # If the payload is a dict that contains the list under a key like 'data', 'items', or 'result', extract it
    if isinstance(data, dict):
        for key in ('data', 'items', 'result', 'rows'):
            if key in data and isinstance(data[key], list):
                data = data[key]
                break
        else:
            # If no list found under common keys, treat the dict as a single item
            data = [data]

    # Ensure we have a list to iterate
    if not isinstance(data, list):
        raise ValueError(f"Unexpected JSON structure: expected list or dict, got {type(data).__name__}")

    # Фильтрация данных — только объекты dict и с count != 0
    filtered_data = [item for item in data if isinstance(item, dict) and item.get('count', 0) != 0]

    # Вывод результата
    print(json.dumps(filtered_data, indent=2, ensure_ascii=False))

except requests.exceptions.RequestException as e:
    print(f"HTTP Request failed: {e}")
except json.JSONDecodeError as e:
    print(f"Failed to parse JSON: {e}")