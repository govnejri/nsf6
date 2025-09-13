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
    
    print("=== ПОЛНЫЙ ОТВЕТ ===")
    print(f"Status Code: {response.status_code}")
    print(f"Headers: {dict(response.headers)}")
    print(f"Raw content: {response.text[:1000]}...")  # Первые 1000 символов
    
    # Парсинг JSON-ответа
    data = response.json()
    print(f"\n=== СТРУКТУРА JSON ===")
    print(f"Type: {type(data)}")
    print(f"Keys: {list(data.keys()) if isinstance(data, dict) else 'Not a dict'}")
    
    if isinstance(data, dict) and 'heatmap' in data:
        heatmap = data['heatmap']
        print(f"Heatmap type: {type(heatmap)}")
        print(f"Heatmap keys: {list(heatmap.keys()) if isinstance(heatmap, dict) else 'Not a dict'}")
        
        if isinstance(heatmap, dict) and 'data' in heatmap:
            tiles = heatmap['data']
            print(f"Tiles count: {len(tiles)}")
            
            # Считаем тайлы с count > 0
            non_zero_tiles = [tile for tile in tiles if isinstance(tile, dict) and tile.get('count', 0) > 0]
            print(f"Non-zero tiles: {len(non_zero_tiles)}")
            
            # Показываем первые несколько тайлов
            print(f"\n=== ПЕРВЫЕ 5 ТАЙЛОВ ===")
            for i, tile in enumerate(tiles[:5]):
                print(f"Tile {i}: {tile}")
            
            if non_zero_tiles:
                print(f"\n=== ПЕРВЫЕ 5 НЕ-НУЛЕВЫХ ТАЙЛОВ ===")
                for i, tile in enumerate(non_zero_tiles[:5]):
                    print(f"Non-zero tile {i}: {tile}")
            
            # Статистика по count
            counts = [tile.get('count', 0) for tile in tiles if isinstance(tile, dict)]
            print(f"\n=== СТАТИСТИКА ===")
            print(f"Total counts: {sum(counts)}")
            print(f"Max count: {max(counts) if counts else 0}")
            print(f"Min count: {min(counts) if counts else 0}")
            print(f"Average count: {sum(counts) / len(counts) if counts else 0}")

except Exception as e:
    print(f"Ошибка: {e}")
    import traceback
    traceback.print_exc()
